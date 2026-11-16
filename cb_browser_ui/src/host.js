// The browser side of the Rust <-> JS boundary.
//
// Everything that touches React, the DOM or the WebGL layer lives here. The
// Rust client (src/js.rs) only passes plain data in; all the
// `update(oldState, {...})` state patching that used to be written inline in
// Rust `js!` blocks is spelled out in one place, in JavaScript, where it belongs.

import update from 'immutability-helper';
import colors from './colors';

function app() {
    return window.cbReactApp;
}

function setState(spec) {
    app().boundSetState(oldState => update(oldState, spec));
}

// ---------------------------------------------------------------- basics

export function log(message) {
    console.log(message);
}

export function error(message) {
    console.error(message);
}

export function panic(message) {
    console.error('Rust WASM error: ' + message);
    const errors = document.getElementById('errors');
    if (errors) {
        errors.className = 'errorsHappened';
        errors.insertAdjacentHTML('beforeend', '<h2>' + message + '</h2>');
    }
}

export function hostname() {
    return window.location.hostname;
}

export function networkSettings() {
    return window.cbNetworkSettings;
}

export function appState() {
    return app().state;
}

export function setTypeIdMapping(mapping) {
    window.cbTypeIdMapping = mapping;
}

export function onFrame() {
    app().onFrame();
}

export function carColors() {
    return colors.carColors;
}

export function trunkColor() {
    return colors.trunks;
}

export function canopyColor() {
    return colors.canopies;
}

// ---------------------------------------------------------------- simulation

export function systemStats(networkingTurns, queueLengths, messageStats) {
    setState({
        system: {
            networkingTurns: { $set: networkingTurns },
            queueLengths: { $set: queueLengths },
            messageStats: { $set: messageStats },
        },
    });
}

export function timeInfo(ticks, time, speed) {
    setState({
        time: {
            ticks: { $set: ticks },
            time: { $set: time },
            speed: { $set: speed },
        },
    });
}

export function logLastEntry() {
    return app().state.debug.logLastEntry;
}

export function currentProjectId() {
    const state = app().state;
    return (state.uiMode == 'planning' && state.planning.currentProject) || null;
}

export function appendLog(entries, text, lastEntry, effectiveLast, effectiveTextStart) {
    if (app().state.debug.logLastEntry == lastEntry) {
        setState({
            debug: {
                logLastEntry: { $apply: n => n + entries.length },
                logEntries: { $push: entries },
                logText: { $apply: t => t + text },
            },
        });
    } else {
        setState({
            debug: {
                logLastEntry: { $set: effectiveLast + entries.length },
                logTextStart: { $set: effectiveTextStart },
                logFirstEntry: { $set: effectiveLast },
                logEntries: { $set: entries },
                logText: { $set: text },
            },
        });
    }
}

// ---------------------------------------------------------------- transport

export function carInstances(instances) {
    setState({ transport: { rendering: { carInstances: { $set: instances } } } });
}

// kind is one of laneAsphaltGroups / laneMarkerGroups / laneMarkerGapGroups
export function laneGroups(kind, groups) {
    setState({ transport: { rendering: { [kind]: { $add: groups } } } });
}

// ---------------------------------------------------------------- households

export function householdInfo(id, core) {
    setState({ households: { householdInfo: { [id]: { $set: { core } } } } });
}

export function inspectedBuilding(households, style) {
    setState({
        households: { inspectedBuildingState: { $set: { households, style } } },
    });
}

// materialMeshes: [[materialName, mesh], ...]
// propInstances: [[propTypeName, instanceBuffer], ...]
export function buildingConstructed(id, materialMeshes, propInstances, position, shape) {
    const meshes = {};
    for (const [material, mesh] of materialMeshes) {
        meshes[material] = { [id]: { $set: mesh } };
    }
    const props = {};
    for (const [propType, instances] of propInstances) {
        props[propType] = { [id]: { $set: instances } };
    }
    setState({
        landUse: { rendering: { buildingMeshes: meshes, buildingProps: props } },
        households: {
            buildingPositions: { [id]: { $set: position } },
            buildingShapes: { [id]: { $set: shape } },
        },
    });
}

export function buildingDestructed(id, materials, propTypes) {
    const meshes = {};
    for (const material of materials) {
        meshes[material] = { $unset: [id] };
    }
    const props = {};
    for (const propType of propTypes) {
        props[propType] = { $unset: [id] };
    }
    setState({
        landUse: { rendering: { buildingMeshes: meshes, buildingProps: props } },
        households: { buildingPositions: { $unset: [id] } },
    });
}

// ---------------------------------------------------------------- vegetation

export function vegetationMeshes(meshes) {
    setState({
        vegetation: {
            trunkMesh: { $set: meshes.trunk },
            mediumCanopyMesh: { $set: meshes.medium },
            smallCanopyMesh: { $set: meshes.small },
            largeCanopyMesh: { $set: meshes.large },
        },
    });
}

export function vegetationInstances(instances) {
    setState({
        vegetation: {
            trunkInstances: { $set: instances.trunk },
            smallCanopyInstances: { $set: instances.small },
            mediumCanopyInstances: { $set: instances.medium },
            largeCanopyInstances: { $set: instances.large },
        },
    });
    console.log('Rebuilt vegetation');
}

// ---------------------------------------------------------------- planning

export function planningStaticMesh(name, mesh) {
    setState({ planning: { rendering: { staticMeshes: { [name]: { $set: mesh } } } } });
}

export function planningMaster(master) {
    setState({ planning: { master: { $set: master } } });
}

export function projectOngoing(id, ongoing) {
    setState({ planning: { projects: { [id]: { ongoing: { $set: ongoing } } } } });
}

export function projectSet(id, project) {
    setState({ planning: { projects: { [id]: { $set: project } } } });
}

export function projectUnset(id) {
    setState({ planning: { projects: { $unset: [id] } } });
}

// d: lanes, lanesMarker, lanesMarkerGaps, zones, zoneOutlines, buildingOutlines, roadInfos
export function planPreview(d) {
    setState({
        planning: {
            rendering: {
                currentPreview: {
                    lanesToConstructGroups: { $add: d.lanes },
                    lanesToConstructMarkerGroups: { $add: d.lanesMarker },
                    lanesToConstructMarkerGapsGroups: { $add: d.lanesMarkerGaps },
                    zoneGroups: d.zones,
                    zoneOutlineGroups: d.zoneOutlines,
                    buildingOutlinesGroup: { $add: d.buildingOutlines },
                },
                roadInfos: { $set: d.roadInfos },
            },
        },
    });
}

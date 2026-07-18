import { useCurrentPod } from '@/context/pods';
import { ALL_POD_STATES, type PodStateType } from '@hyped/telemetry-constants';
import { useMemo } from 'react';
import ReactFlow, { Background, Position } from 'reactflow';
import 'reactflow/dist/style.css';
import { arrow } from './edges';
import { ActiveNode, FailureNode, NeutralNode, PassiveNode } from './nodes';
import './styles.css';
import type { CustomEdgeType, CustomNodeType } from './types';
import { getNodeType } from './utils';

const runStates = [
	{
		id: 'idle',
		label: 'Idle',
		state: ALL_POD_STATES.IDLE,
		position: { x: 0, y: 0 },
	},
	{
		id: 'setup-motor',
		label: 'Motor Setup',
		state: ALL_POD_STATES.SETUP_MOTOR,
		position: { x: 240, y: 0 },
	},
	{
		id: 'precharge',
		label: 'Precharge',
		state: ALL_POD_STATES.PRECHARGE,
		position: { x: 480, y: 0 },
	},
	{
		id: 'hv-active',
		label: 'HV Active',
		state: ALL_POD_STATES.HV_ACTIVE,
		position: { x: 720, y: 0 },
	},
	{
		id: 'ready-for-propulsion',
		label: 'Ready for Propulsion',
		state: ALL_POD_STATES.READY_FOR_PROPULSION,
		position: { x: 960, y: 0 },
	},
	{
		id: 'accelerate',
		label: 'Accelerate',
		state: ALL_POD_STATES.ACCELERATE,
		position: { x: 1200, y: 0 },
	},
	{
		id: 'brake',
		label: 'Brake',
		state: ALL_POD_STATES.BRAKE,
		position: { x: 1440, y: 0 },
	},
	{
		id: 'stopped',
		label: 'Stopped',
		state: ALL_POD_STATES.STOPPED,
		position: { x: 1680, y: 0 },
	},
] as const;

const emergencyNode = {
	id: 'emergency',
	label: 'Emergency',
	state: ALL_POD_STATES.EMERGENCY,
	position: { x: 840, y: 220 },
} as const;

const maintenanceNode = {
	id: 'maintenance',
	label: 'Maintenance',
	state: ALL_POD_STATES.MAINTENANCE,
	position: { x: 0, y: 220 },
} as const;

export function StateMachine() {
	const nodeTypes = useMemo(
		() => ({
			FailureNode,
			PassiveNode,
			ActiveNode,
			NeutralNode,
		}),
		[],
	);

	const {
		pod: { podState: currentState },
	} = useCurrentPod();

	const nodes: CustomNodeType[] = useMemo(
		() =>
			[...runStates, maintenanceNode, emergencyNode].map((node) => ({
				id: node.id,
				data: {
					label: node.label,
					sourcePositions: [
						{ position: Position.Right, id: 'right' },
						{ position: Position.Bottom, id: 'bottom' },
					],
					targetPositions: [
						{ position: Position.Left, id: 'left' },
						{ position: Position.Top, id: 'top' },
					],
					active: currentState === node.state,
				},
				position: node.position,
				type: getNodeType(node.state as PodStateType),
			})),
		[currentState],
	);

	const edges: CustomEdgeType[] = useMemo(() => {
		const runEdges = runStates.slice(0, -1).map((state, index) => ({
			id: `${state.id}-${runStates[index + 1].id}`,
			source: state.id,
			target: runStates[index + 1].id,
			sourceHandle: 'right',
			targetHandle: 'left',
			type: 'smoothstep',
			pathOptions: { borderRadius: 20 },
			markerEnd: arrow,
		}));

		const maintenanceEdges: CustomEdgeType[] = [
			{
				id: 'idle-maintenance',
				source: 'idle',
				target: maintenanceNode.id,
				sourceHandle: 'bottom',
				targetHandle: 'top',
				type: 'smoothstep',
				pathOptions: { borderRadius: 20 },
				markerEnd: arrow,
			},
		];

		const activeNode = nodes.find((node) => node.data.active);
		if (!activeNode || activeNode.id === emergencyNode.id)
			return [...runEdges, ...maintenanceEdges];

		return [
			...runEdges,
			...maintenanceEdges,
			{
				id: `${activeNode.id}-emergency`,
				source: activeNode.id,
				target: emergencyNode.id,
				sourceHandle: 'bottom',
				targetHandle: 'top',
				type: 'smoothstep',
				pathOptions: { borderRadius: 20 },
				markerEnd: arrow,
			},
		];
	}, [nodes]);

	return (
		<div className="h-full flex flex-col justify-center items-center">
			<div className="h-full w-full">
				<ReactFlow
					nodes={nodes}
					edges={edges}
					nodeTypes={nodeTypes}
					nodesDraggable={false}
					nodesConnectable={false}
					fitView
				>
					<Background />
				</ReactFlow>
			</div>
		</div>
	);
}

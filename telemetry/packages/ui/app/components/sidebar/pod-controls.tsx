import { Button } from '@/components/ui/button';
import { usePod } from '@/context/pods';
import { CONTROLS, sendControlMessage } from '@/lib/controls';
import { cn } from '@/lib/utils';
import { ALL_POD_STATES } from '@hyped/telemetry-constants';
import {
	Gauge,
	type LucideIcon,
	PlugZap,
	Rocket,
	ShieldCheck,
	Siren,
} from 'lucide-react';

/**
 * Displays the pod controls.
 * @param podId The ID of the pod to display the controls of.
 * @param show Whether or not to show the controls. This is used to keep the state of the controls when the podId changes (rather than unmounting and remounting the component).
 * @returns The pod controls.
 */
export const PodControls = ({
	podId,
	show,
}: {
	podId: string;
	show: boolean;
}) => {
	const { podState } = usePod(podId);
	const controls = [
		{
			label: 'Motor Setup',
			control: CONTROLS.MOTOR_SETUP,
			icon: PlugZap,
			enabled:
				podState === ALL_POD_STATES.IDLE ||
				podState === ALL_POD_STATES.UNKNOWN,
			className: 'bg-blue-700 hover:bg-blue-800',
		},
		{
			label: 'Precharge',
			control: CONTROLS.PRECHARGE,
			icon: Gauge,
			enabled: podState === ALL_POD_STATES.SETUP_MOTOR,
			className: 'bg-amber-600 hover:bg-amber-700',
		},
		{
			label: 'Ready Pod',
			control: CONTROLS.READY_FOR_PROPULSION,
			icon: ShieldCheck,
			enabled: podState === ALL_POD_STATES.PRECHARGE,
			className: 'bg-emerald-700 hover:bg-emerald-800',
		},
		{
			label: 'Accelerate',
			control: CONTROLS.ACCELERATE,
			icon: Rocket,
			enabled: podState === ALL_POD_STATES.READY_FOR_PROPULSION,
			className: 'bg-green-700 hover:bg-green-800',
		},
	];

	return (
		<div className={cn('mt-2 space-y-4', show ? 'block' : 'hidden')}>
			<div className="flex flex-col gap-3">
				{controls.map((control) => (
					<PodControlButton key={control.control} podId={podId} {...control} />
				))}
				<EmergencyStopButton podId={podId} />
			</div>
		</div>
	);
};

const PodControlButton = ({
	podId,
	label,
	control,
	icon: Icon,
	enabled,
	className,
}: {
	podId: string;
	label: string;
	control: (typeof CONTROLS)[keyof typeof CONTROLS];
	icon: LucideIcon;
	enabled: boolean;
	className: string;
}) => (
	<Button
		className={cn(
			'w-full px-2 py-6 rounded-md shadow-lg transition text-white font-bold flex gap-2 disabled:bg-gray-400 disabled:cursor-not-allowed',
			className,
		)}
		disabled={!enabled}
		onClick={() => void sendControlMessage(podId, control)}
	>
		<Icon /> {label}
	</Button>
);

export const EmergencyStopButton = ({
	podId,
	className,
}: {
	podId: string;
	className?: string;
}) => (
	<Button
		className={cn(
			'px-2 py-6 rounded-md shadow-lg transition text-white font-bold flex gap-2',
			'bg-red-700 hover:bg-red-800',
			className,
		)}
		onClick={() => void sendControlMessage(podId, CONTROLS.EMERGENCY_STOP)}
	>
		<Siren /> EMERGENCY STOP
	</Button>
);

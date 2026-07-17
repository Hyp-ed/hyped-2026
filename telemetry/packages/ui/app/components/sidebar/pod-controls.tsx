import { Button } from '@/components/ui/button';
import { CONTROLS, sendControlMessage } from '@/lib/controls';
import { cn } from '@/lib/utils';
import { Siren } from 'lucide-react';

/**
 * Displays the emergency pod control in the sidebar.
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
	return (
		<div className={cn('mt-2', show ? 'block' : 'hidden')}>
			<EmergencyStopButton podId={podId} className="w-full" />
		</div>
	);
};

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

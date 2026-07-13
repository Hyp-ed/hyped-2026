import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { usePods } from '@/context/pods';
import { ALL_POD_STATES, type PodId } from '@hyped/telemetry-constants';
import { useEffect, useMemo, useState } from 'react';
import { DialogHeader } from './ui/dialog';

type Acknowledgements = Partial<Record<PodId, boolean>>;

export const HighPowerPrompt = () => {
	const { pods } = usePods();
	const [acknowledged, setAcknowledged] = useState<Acknowledgements>({});

	const promptPod = useMemo(
		() =>
			Object.values(pods).find(
				(pod) =>
					pod.podState === ALL_POD_STATES.SETUP_MOTOR &&
					pod.controlStatus.canPrecharge &&
					!acknowledged[pod.id],
			),
		[pods, acknowledged],
	);

	useEffect(() => {
		setAcknowledged((current) => {
			let changed = false;
			const next = { ...current };
			for (const pod of Object.values(pods)) {
				if (
					pod.podState !== ALL_POD_STATES.SETUP_MOTOR ||
					!pod.controlStatus.canPrecharge
				) {
					if (next[pod.id]) {
						delete next[pod.id];
						changed = true;
					}
				}
			}
			return changed ? next : current;
		});
	}, [pods]);

	return (
		<AlertDialog open={Boolean(promptPod)}>
			<AlertDialogContent>
				<DialogHeader>
					<AlertDialogTitle>Start High Power</AlertDialogTitle>
				</DialogHeader>
				<AlertDialogDescription>
					Motor setup is complete for {promptPod?.name}. Start the High Power
					system before continuing to precharge.
				</AlertDialogDescription>
				<AlertDialogFooter>
					<AlertDialogAction
						onClick={() => {
							if (!promptPod) return;
							setAcknowledged((current) => ({
								...current,
								[promptPod.id]: true,
							}));
						}}
					>
						Acknowledge
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	);
};

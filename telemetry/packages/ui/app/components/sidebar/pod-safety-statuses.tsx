import { usePod } from '@/context/pods';
import { cn } from '@/lib/utils';

const StatusRow = ({
	label,
	status,
	fault,
}: {
	label: string;
	status: string;
	fault?: boolean;
}) => (
	<div className="flex items-center justify-between gap-3 text-sm">
		<span>{label}</span>
		<span className="flex items-center gap-2 font-mono font-semibold">
			<span
				className={cn(
					'h-2 w-2 rounded-full',
					status === 'UNKNOWN' && 'bg-orange-500',
					status !== 'UNKNOWN' && fault && 'bg-red-500',
					status !== 'UNKNOWN' && !fault && 'bg-green-500',
				)}
			/>
			{status}
		</span>
	</div>
);

export const PodSafetyStatuses = ({ podId }: { podId: string }) => {
	const { imdStatus, brakeClampStatus } = usePod(podId);

	return (
		<div className="flex flex-col gap-2 rounded-md bg-openmct-dark-gray px-3 py-2">
			<StatusRow
				label="IMD Status"
				status={imdStatus}
				fault={imdStatus === 'FAULT'}
			/>
			<StatusRow label="Brake Clamp Status" status={brakeClampStatus} />
		</div>
	);
};

import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from '@/components/ui/card';
import { usePod } from '@/context/pods';
import { cn } from '@/lib/utils';
import { ShieldCheck } from 'lucide-react';

const HvalLamp = ({
	label,
	active,
	colour,
}: {
	label: string;
	active: boolean | null;
	colour: 'red' | 'green';
}) => (
	<div className="flex items-center gap-3">
		<span
			className={cn(
				'h-8 w-8 shrink-0 rounded-full border-2 transition-colors',
				active === null && 'border-gray-500 bg-gray-700',
				colour === 'red' && active === false && 'border-red-950 bg-red-950',
				colour === 'red' && active === true && 'border-red-300 bg-red-500',
				colour === 'green' && active === false && 'border-green-950 bg-green-950',
				colour === 'green' && active === true && 'border-green-300 bg-green-500',
			)}
			aria-label={`${label}: ${active === null ? 'unknown' : active ? 'on' : 'off'}`}
		/>
		<div>
			<p className="font-semibold">{label}</p>
			<p className="text-xs text-muted-foreground">
				{active === null ? 'UNKNOWN' : active ? 'ON' : 'OFF'}
			</p>
		</div>
	</div>
);

export const SafetyStatuses = ({ podId }: { podId: string }) => {
	const { imdStatus, hvalRedActive, hvalGreenActive } = usePod(podId);

	return (
		<Card className="w-full border-none">
			<CardHeader className="pb-2">
				<CardTitle className="flex gap-2">
					HVAL and IMD
				</CardTitle>
			</CardHeader>
			<CardContent className="flex items-center justify-around gap-6">
				<div className="flex gap-6">
					<HvalLamp label="HVAL Red" active={hvalRedActive} colour="red" />
					<HvalLamp
						label="HVAL Green"
						active={hvalGreenActive}
						colour="green"
					/>
				</div>
				<div className="flex items-center gap-3">
					<span
						className={cn(
							'h-4 w-4 rounded-full',
							imdStatus === 'HEALTHY' && 'bg-green-500',
							imdStatus === 'FAULT' && 'bg-red-500',
							imdStatus === 'UNKNOWN' && 'bg-orange-500',
						)}
					/>
					<div>
						<p className="font-semibold">IMD</p>
						<p className="text-xs text-muted-foreground">{imdStatus}</p>
					</div>
				</div>
			</CardContent>
		</Card>
	);
};

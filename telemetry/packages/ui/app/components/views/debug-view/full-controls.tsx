import { Button } from '@/components/ui/button';
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from '@/components/ui/card';
import { CONTROLS, sendControlMessage } from '@/lib/controls';
import { cn } from '@/lib/utils';
import {
	ChevronsDown,
	ChevronsUp,
	Gauge,
	Rocket,
	Settings2,
	ShieldCheck,
	Siren,
	Wrench,
} from 'lucide-react';
import React from 'react';

/**
 * Full granular pod controls. Used in the debug view.
 * @param podId The ID of the pod to display the controls of.
 * @returns A Card component displaying the full granular pod controls.
 */
export const FullControls = ({ podId }: { podId: string }) => {
	return (
		<Card className="w-full border-none">
			<CardHeader>
				<CardTitle className="flex gap-2">
					<Settings2 /> Controls
				</CardTitle>
				<CardDescription>Granular pod controls</CardDescription>
			</CardHeader>
			<CardContent>
				<div className="flex flex-row gap-3 overflow-x-auto">
					<div className="flex flex-row gap-3 shrink-0">
						<ButtonLabel>Run sequence</ButtonLabel>
						<ButtonPair>
							<LeftButton
								onClick={() =>
									void sendControlMessage(podId, CONTROLS.MOTOR_SETUP)
								}
							>
								Motor Setup <Wrench size={16} />
							</LeftButton>
							<RightButton
								onClick={() => void sendControlMessage(podId, CONTROLS.PRECHARGE)}
							>
								Precharge <Gauge size={16} />
							</RightButton>
						</ButtonPair>
						<ButtonPair>
							<LeftButton
								onClick={() =>
									void sendControlMessage(
										podId,
										CONTROLS.READY_FOR_PROPULSION,
									)
								}
							>
								Ready Pod <ShieldCheck size={16} />
							</LeftButton>
							<RightButton
								onClick={() => void sendControlMessage(podId, CONTROLS.ACCELERATE)}
							>
								Accelerate <ChevronsUp size={16} />
							</RightButton>
						</ButtonPair>
					</div>
					<ControlButton
						className="bg-openmct-dark-gray hover:bg-openmct-light-gray flex gap-2"
						onClick={() => void sendControlMessage(podId, CONTROLS.STOP)}
					>
						Brake <ChevronsDown size={16} />
					</ControlButton>
					<ControlButton
						className="bg-red-700 hover:bg-red-800 text-white flex gap-2"
						onClick={() =>
							void sendControlMessage(podId, CONTROLS.EMERGENCY_STOP)
						}
					>
						E-Stop <Siren size={16} />
					</ControlButton>
				</div>
			</CardContent>
		</Card>
	);
};

const ButtonLabel = ({ children }: { children: React.ReactNode }) => (
	<p className="text-sm font-bold self-center whitespace-nowrap">{children}</p>
);

/**
 * Sets up the default styling for a pair of buttons. (space-x-0.5)
 */
const ButtonPair = ({ children }: { children: React.ReactNode }) => {
	return <div className="flex flex-row gap-3">{children}</div>;
};

/**
 * Sets up the default styling for a control button.
 */
const ControlButton = React.forwardRef<
	HTMLButtonElement,
	React.ButtonHTMLAttributes<HTMLButtonElement>
>((props, ref) => {
	return (
		<Button
			// @ts-ignore
			ref={ref}
			{...props}
			className={cn(
				'bg-openmct-dark-gray hover:bg-openmct-light-gray w-40 shrink-0 py-6',
				props.className,
			)}
		/>
	);
});
ControlButton.displayName = 'ControlButton';

/**
 * Sets up the default styling for a control button on the left side of a pair. (rounded left)
 */
const LeftButton = React.forwardRef<
	HTMLButtonElement,
	React.ButtonHTMLAttributes<HTMLButtonElement>
>((props, ref) => {
	return (
		<ControlButton
			// @ts-ignore
			ref={ref}
			{...props}
			className={cn(
				'bg-openmct-dark-gray hover:bg-openmct-light-gray pr-2 flex gap-2',
				props.className,
			)}
		/>
	);
});
LeftButton.displayName = 'LeftButton';

/**
 * Sets up the default styling for a control button on the right side of a pair. (rounded right)
 */
const RightButton = React.forwardRef<
	HTMLButtonElement,
	React.ButtonHTMLAttributes<HTMLButtonElement>
>((props, ref) => {
	return (
		<ControlButton
			// @ts-ignore
			ref={ref}
			{...props}
			className={cn(
				'bg-openmct-dark-gray hover:bg-openmct-light-gray pl-2 flex gap-2',
				props.className,
			)}
		/>
	);
});
RightButton.displayName = 'RightButton';

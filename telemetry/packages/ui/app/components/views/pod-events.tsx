import { type Log, useLiveLogs } from '@/context/live-logs';
import { format } from 'date-fns';
import { Radio, X } from 'lucide-react';
import { useEffect } from 'react';
import { Button } from '../ui/button';

const POD_EVENTS_CONTEXT = 'PodEvents';

export const PodEvents = () => {
	const { clearAll, isConnected, logs } = useLiveLogs();
	const podEvents = logs.filter((log) => log.context === POD_EVENTS_CONTEXT);

	useEffect(() => {
		const element = document.getElementById('pod-events');
		if (element) {
			element.scrollTop = element.scrollHeight;
		}
	}, [podEvents]);

	return (
		<div className="h-full p-12 space-y-6">
			<div className="flex items-center justify-between gap-4">
				<div className="flex items-center gap-3">
					<Radio className="h-5 w-5 text-green-400" />
					<div>
						<h1 className="text-white text-2xl font-semibold">Pod Events</h1>
						<p className="text-sm text-gray-400">
							{podEvents.length} retained pod messages
						</p>
					</div>
				</div>
				<div className="flex items-center gap-3">
					<div className="flex items-center gap-2">
						<div
							className={`h-2 w-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`}
						/>
						<span
							className={`text-sm italic ${isConnected ? 'text-green-500' : 'text-red-500'}`}
						>
							{isConnected ? 'Connected' : 'Disconnected'}
						</span>
					</div>
					<Button variant="outline" onClick={clearAll} className="flex gap-2">
						<X className="h-4 w-4 opacity-50" />
						Clear
					</Button>
				</div>
			</div>

			<div
				id="pod-events"
				className="h-[calc(100%-88px)] overflow-y-auto rounded border border-openmct-dark-gray bg-black/30 font-logs scrollbar-track-transparent scrollbar-thumb-openmct-dark-gray scrollbar-thin scrollbar-thumb-rounded-full"
			>
				{podEvents.length > 0 ? (
					podEvents.map((event, index) => (
						<PodEventRow
							// biome-ignore lint/suspicious/noArrayIndexKey: logs do not have stable IDs
							key={index}
							event={event}
						/>
					))
				) : (
					<div className="p-6 text-center text-gray-400">
						Waiting for pod events...
					</div>
				)}
			</div>
		</div>
	);
};

const PodEventRow = ({ event }: { event: Log }) => {
	const time = format(new Date(event.timestamp), 'HH:mm:ss.SSS');

	return (
		<div className="grid grid-cols-[110px_1fr] gap-4 border-openmct-dark-gray border-b px-4 py-2 text-sm">
			<span className="text-yellow-400">{time}</span>
			<span className="break-words text-green-300">{event.message}</span>
		</div>
	);
};

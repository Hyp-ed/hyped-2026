import { log } from '@/lib/logger';
import { http } from 'openmct/core/http';
import toast from 'react-hot-toast';

/**
 * Defines the controls that can be sent to a pod.
 */
export const CONTROLS = {
	IDLE: 'idle',
	MAINTENANCE: 'maintenance',
	RESET_EMERGENCY: 'reset-emergency',
	MOTOR_SETUP: 'setup_motor',
	PRECHARGE: 'precharge',
	READY_FOR_PROPULSION: 'ready-for-propulsion',
	ACCELERATE: 'accelerate',
	STOP: 'stop',
	EMERGENCY_STOP: 'emergency-stop',
} as const;

export type Control = (typeof CONTROLS)[keyof typeof CONTROLS];

/**
 * Sends a control message to a pod.
 * @param podId The ID of the pod to send the control to.
 * @param control The control message to send.
 */
export const sendControlMessage = async (podId: string, control: Control) => {
	log(`Sending control ${control} to pod ${podId}`, podId);
	toast(`Sending control ${control} to pod ${podId}`);
	const url = `pods/${podId}/controls/${control}`;
	await http.post(url);
};

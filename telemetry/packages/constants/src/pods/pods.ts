
import { PodSchema } from '@hyped/telemetry-types';
import * as YAML from 'yaml';
import { z } from 'zod';
import { telemetryTypes } from './types';
import { podsYamlContent } from './pods-data.generated';





const RawPodsSchema = z.object({
	pods: z.record(
		z.object({
			label: z.string(),
			mode: z.enum(['ALL_SYSTEMS_ON', 'LEVITATION_ONLY', 'LIM_ONLY']),
			measurements: z.record(z.object({}).passthrough()),
			statuses: z.record(z.object({}).passthrough()),
		}),
	),
});

// We also want to check the 'type' field of each measurement and status is one of the object types
// It would be unwise to add this directly to the schema (in the types package) because it would
// break the circular dependency between the types and constants packages.
type MeasurementType = (typeof telemetryTypes)[number];
// const validateType = (
// 	ctx: z.RefinementCtx,
// 	items: Record<string, unknown>,
// 	type: 'measurements' | 'statuses',
// ) => {
// 	for (const [id, item] of Object.entries(items)) {
// 		const itemType = (item as { type: string }).type;
// 		if (!telemetryTypes.includes(itemType as MeasurementType)) {
// 			ctx.addIssue({
// 				code: z.ZodIssueCode.custom,
// 				message: `Invalid ${type.slice(0, -1)} type "${itemType}"`,
// 				path: [type, id, 'type'],
// 			});
// 		}
// 	}
// };
// Validate types on the raw data BEFORE PodSchema strips them out
const validateRawPodData = (podData: any) => {
	for (const [id, measurement] of Object.entries(podData.measurements)) {
		const itemType = (measurement as any).kind;
		if (!telemetryTypes.includes(itemType as MeasurementType)) {
			throw new Error(`Invalid measurement type "${itemType}" for ${id}`);
		}
	}
	for (const [id, status] of Object.entries(podData.statuses)) {
		const itemType = (status as any).kind;
		if (!telemetryTypes.includes(itemType as MeasurementType)) {
			throw new Error(`Invalid status type "${itemType}" for ${id}`);
		}
	}
};

const yamlData = RawPodsSchema.parse(YAML.parse(podsYamlContent));



export const pods = Object.fromEntries(
	Object.entries(yamlData.pods).map(([podId, podData]) => {
		// Validate types BEFORE PodSchema strips them
		validateRawPodData(podData);
		
		return [
			podId,
			PodSchema.parse({  // Changed from ExtendedPodSchema
				id: podId,
				...podData,
				measurements: Object.fromEntries(
					Object.entries(podData.measurements).map(([id, measurement]) => [
						id,
						{ id, ...measurement },
					]),
				),
				statuses: Object.fromEntries(
					Object.entries(podData.statuses).map(([id, status]) => [
						id,
						{ id, ...status },
					]),
				),
			}),
		];
	}),
);


export const podIds = Object.keys(pods);

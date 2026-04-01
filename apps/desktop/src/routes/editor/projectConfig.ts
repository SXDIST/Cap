import type {
	AspectRatio,
	ScreenMovementSpring,
	ZoomSegment,
} from "~/utils/tauri";

export type RGBColor = [number, number, number];

export const DEFAULT_GRADIENT_FROM = [71, 133, 255] satisfies RGBColor;
export const DEFAULT_GRADIENT_TO = [255, 71, 102] satisfies RGBColor;

export const ASPECT_RATIOS = {
	wide: { name: "Wide", ratio: [16, 9] },
	vertical: { name: "Vertical", ratio: [9, 16] },
	square: { name: "Square", ratio: [1, 1] },
	classic: { name: "Classic", ratio: [4, 3] },
	tall: { name: "Tall", ratio: [3, 4] },
} satisfies Record<AspectRatio, { name: string; ratio: [number, number] }>;

export type ZoomAnimationPreset =
	| "cinematic"
	| "balanced"
	| "focus"
	| "responsive"
	| "custom";

export type ZoomAnimationPresetValues = {
	screenMovementSpring: ScreenMovementSpring;
	screenMotionBlur: number;
	edgeSnapRatio: number;
	instantAnimation: boolean;
};

export const DEFAULT_SCREEN_MOVEMENT_SPRING = {
	stiffness: 120,
	damping: 14,
	mass: 1,
} satisfies ScreenMovementSpring;

export const ZOOM_ANIMATION_PRESET_OPTIONS = [
	{
		value: "cinematic",
		label: "Cinematic",
		description: "Softer camera follow with a polished, slower catch-up.",
		preset: {
			screenMovementSpring: { stiffness: 150, damping: 17, mass: 1.02 },
			screenMotionBlur: 0.3,
			edgeSnapRatio: 0.16,
			instantAnimation: false,
		},
	},
	{
		value: "balanced",
		label: "Balanced",
		description: "Smooth default zoom with natural cursor follow.",
		preset: {
			screenMovementSpring: { stiffness: 195, damping: 18, mass: 0.92 },
			screenMotionBlur: 0.24,
			edgeSnapRatio: 0.14,
			instantAnimation: false,
		},
	},
	{
		value: "focus",
		label: "Focus",
		description: "Smart edge-triggered camera shift with stronger tracking blur.",
		preset: {
			screenMovementSpring: { stiffness: 156, damping: 13.2, mass: 1.04 },
			screenMotionBlur: 0.42,
			edgeSnapRatio: 0.22,
			instantAnimation: false,
		},
	},
	{
		value: "responsive",
		label: "Responsive",
		description: "Fast, clean follow tuned for quick cursor movement.",
		preset: {
			screenMovementSpring: { stiffness: 265, damping: 21, mass: 0.82 },
			screenMotionBlur: 0.2,
			edgeSnapRatio: 0.1,
			instantAnimation: false,
		},
	},
	{
		value: "custom",
		label: "Custom",
		description: "Tune screen follow manually.",
	},
] satisfies Array<{
	value: ZoomAnimationPreset;
	label: string;
	description: string;
	preset?: ZoomAnimationPresetValues;
}>;

const SCREEN_MOVEMENT_SPRING_TOLERANCE = {
	stiffness: 1,
	damping: 0.2,
	mass: 0.05,
} as const;

const ZOOM_ANIMATION_PRESET_TOLERANCE = {
	screenMotionBlur: 0.03,
	edgeSnapRatio: 0.02,
} as const;

export function normalizeScreenMovementSpring(
	spring?: ScreenMovementSpring | null,
): ScreenMovementSpring {
	return {
		stiffness: spring?.stiffness ?? DEFAULT_SCREEN_MOVEMENT_SPRING.stiffness,
		damping: spring?.damping ?? DEFAULT_SCREEN_MOVEMENT_SPRING.damping,
		mass: spring?.mass ?? DEFAULT_SCREEN_MOVEMENT_SPRING.mass,
	};
}

function getZoomPresetByScreenSpring(
	screenMovementSpring?: ScreenMovementSpring | null,
) {
	const normalizedSpring = normalizeScreenMovementSpring(screenMovementSpring);

	return ZOOM_ANIMATION_PRESET_OPTIONS.find(
		(option) =>
			option.preset &&
			Math.abs(
				option.preset.screenMovementSpring.stiffness -
					normalizedSpring.stiffness,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.stiffness &&
			Math.abs(
				option.preset.screenMovementSpring.damping - normalizedSpring.damping,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.damping &&
			Math.abs(
				option.preset.screenMovementSpring.mass - normalizedSpring.mass,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.mass,
	);
}

export function getZoomAnimationPreset(
	screenMovementSpring?: ScreenMovementSpring | null,
	screenMotionBlur?: number | null,
	zoomSegments?: ZoomSegment[] | null,
): ZoomAnimationPreset | null {
	const matchedBySpring = getZoomPresetByScreenSpring(screenMovementSpring);
	const representativeSegment = zoomSegments?.[0];
	const values = {
		screenMovementSpring: normalizeScreenMovementSpring(screenMovementSpring),
		screenMotionBlur:
			screenMotionBlur ?? matchedBySpring?.preset?.screenMotionBlur ?? 0.3,
		edgeSnapRatio:
			representativeSegment?.edgeSnapRatio ??
			matchedBySpring?.preset?.edgeSnapRatio ??
			0.25,
		instantAnimation:
			representativeSegment?.instantAnimation ??
			matchedBySpring?.preset?.instantAnimation ??
			false,
	};

	const preset = ZOOM_ANIMATION_PRESET_OPTIONS.find(
		(option) =>
			option.preset &&
			Math.abs(
				option.preset.screenMovementSpring.stiffness -
					values.screenMovementSpring.stiffness,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.stiffness &&
			Math.abs(
				option.preset.screenMovementSpring.damping -
					values.screenMovementSpring.damping,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.damping &&
			Math.abs(
				option.preset.screenMovementSpring.mass -
					values.screenMovementSpring.mass,
			) <= SCREEN_MOVEMENT_SPRING_TOLERANCE.mass &&
			Math.abs(option.preset.screenMotionBlur - values.screenMotionBlur) <=
				ZOOM_ANIMATION_PRESET_TOLERANCE.screenMotionBlur &&
			Math.abs(option.preset.edgeSnapRatio - values.edgeSnapRatio) <=
				ZOOM_ANIMATION_PRESET_TOLERANCE.edgeSnapRatio &&
			option.preset.instantAnimation === values.instantAnimation,
	);

	return preset?.value ?? null;
}

export function getZoomSegmentAnimationDefaults(
	screenMovementSpring?: ScreenMovementSpring | null,
) {
	const matchedPreset = getZoomPresetByScreenSpring(screenMovementSpring);

	return {
		instantAnimation: matchedPreset?.preset?.instantAnimation ?? false,
		edgeSnapRatio: matchedPreset?.preset?.edgeSnapRatio ?? 0.25,
	};
}

export function applyZoomAnimationDefaults(
	zoomSegments: ZoomSegment[],
	screenMovementSpring?: ScreenMovementSpring | null,
): ZoomSegment[] {
	const defaults = getZoomSegmentAnimationDefaults(screenMovementSpring);

	return zoomSegments.map((segment) => ({
		...segment,
		instantAnimation: defaults.instantAnimation,
		edgeSnapRatio: defaults.edgeSnapRatio,
	}));
}

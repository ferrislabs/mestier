import {
	Activity,
	Boxes,
	GitBranch,
	Globe,
	type LucideIcon,
	MousePointerClick,
	Wrench,
	Zap,
} from 'lucide-react'

export interface ConnectorAppearance {
	icon: LucideIcon
	hue: number
}

const BY_FAMILY: Record<string, ConnectorAppearance> = {
	mestier: { icon: Activity, hue: 165 },
	http: { icon: Globe, hue: 245 },
	odoo: { icon: Boxes, hue: 300 },
	flow: { icon: GitBranch, hue: 80 },
}

export const TRIGGER_APPEARANCE: ConnectorAppearance = { icon: Zap, hue: 30 }

const FALLBACK: ConnectorAppearance = { icon: Wrench, hue: 220 }

export function connectorAppearance(
	family: string | undefined,
): ConnectorAppearance {
	return (family && BY_FAMILY[family]) || FALLBACK
}

const MANUAL_TRIGGER_APPEARANCE: ConnectorAppearance = {
	icon: MousePointerClick,
	hue: 220,
}

const EVENT_TRIGGER_APPEARANCE: ConnectorAppearance = {
	icon: Activity,
	hue: 165,
}

export function triggerAppearance(manual: boolean): ConnectorAppearance {
	return manual ? MANUAL_TRIGGER_APPEARANCE : EVENT_TRIGGER_APPEARANCE
}

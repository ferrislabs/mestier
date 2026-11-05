import {
	Activity,
	Boxes,
	GitBranch,
	Globe,
	type LucideIcon,
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

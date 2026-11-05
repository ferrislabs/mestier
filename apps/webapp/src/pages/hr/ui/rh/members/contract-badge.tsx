import { cn } from '#/lib/utils'
import { isFixedTerm } from '#/pages/hr/lib/rh/contract-draft'
import {
	CONTRACT_LABELS,
	type ContractPhase,
} from '#/pages/hr/lib/rh/contracts'
import type { ContractType } from '#/pages/hr/lib/rh/types'

export function ContractBadge({
	contract,
	className,
}: {
	contract: ContractType
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center whitespace-nowrap rounded-sm px-1.5 font-medium text-[11px]',
				isFixedTerm(contract)
					? 'bg-state-today/12 text-state-today'
					: 'bg-muted text-muted-foreground',
				className,
			)}
		>
			{CONTRACT_LABELS[contract]}
		</span>
	)
}

export function ArchivedBadge({ className }: { className?: string }) {
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center whitespace-nowrap rounded-sm bg-muted px-1.5 font-medium text-[11px] text-muted-foreground',
				className,
			)}
		>
			Archivé
		</span>
	)
}

const PHASES: Record<ContractPhase, { label: string; tone: string }> = {
	upcoming: { label: 'À venir', tone: 'bg-state-today/12 text-state-today' },
	current: {
		label: 'En cours',
		tone: 'bg-state-progress/12 text-state-progress',
	},
	ended: { label: 'Terminé', tone: 'bg-muted text-muted-foreground' },
}

export function ContractPhaseBadge({
	phase,
	className,
}: {
	phase: ContractPhase
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center whitespace-nowrap rounded-sm px-1.5 font-medium text-[11px]',
				PHASES[phase].tone,
				className,
			)}
		>
			{PHASES[phase].label}
		</span>
	)
}

export const PHASE_DOT: Record<ContractPhase, string> = {
	upcoming: 'bg-state-today',
	current: 'bg-state-progress',
	ended: 'bg-muted-foreground/40',
}

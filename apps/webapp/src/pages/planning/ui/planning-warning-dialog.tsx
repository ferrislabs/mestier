import { AlertTriangle } from 'lucide-react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import {
	type Warning,
	warningDetail,
	warningTitle,
} from '#/pages/planning/lib/warnings'

export interface PlanningWarningDialogProps {
	open: boolean
	warnings: Warning[]
	isPending: boolean
	/** Set when a previous confirm attempt failed — the dialog stays open so the user can retry rather than silently losing the gesture. */
	error?: string | null
	onConfirm: () => void
	onCancel: () => void
}

/**
 * The single dialog every risky gesture on the planning grid funnels through
 * — a drop onto someone on leave, outside their hours, or already busy on
 * another chantier (see the planning design doc's "Avertissements" section:
 * one dialog per gesture, fed by one
 * `warnings` list, never several stacked). It never blocks: confirming
 * always applies the mutation, whatever the warnings say — the API doesn't
 * refuse an assignment for unavailability, and neither does this dialog.
 */
export function PlanningWarningDialog({
	open,
	warnings,
	isPending,
	error,
	onConfirm,
	onCancel,
}: PlanningWarningDialogProps) {
	return (
		<AlertDialog
			open={open}
			onOpenChange={(next) => {
				if (!next) onCancel()
			}}
		>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>
						Confirmer malgré les avertissements ?
					</AlertDialogTitle>
					<AlertDialogDescription>
						Cette action n’est pas bloquée, mais voici ce qu’il faut savoir
						avant de confirmer.
					</AlertDialogDescription>
				</AlertDialogHeader>

				<ul className="flex flex-col gap-2">
					{warnings.map((warning, index) => (
						<li
							// biome-ignore lint/suspicious/noArrayIndexKey: warnings have no stable id, order is stable within a render
							key={index}
							className="flex items-start gap-2 rounded-lg border border-warning/40 bg-warning-soft px-3 py-2 text-sm text-warning"
						>
							<AlertTriangle className="mt-0.5 size-4 shrink-0" />
							<div>
								<p className="font-medium">{warningTitle(warning)}</p>
								{warningDetail(warning) ? (
									<p className="text-xs opacity-90">{warningDetail(warning)}</p>
								) : null}
							</div>
						</li>
					))}
				</ul>

				{error ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
						{error}
					</p>
				) : null}

				<AlertDialogFooter>
					<AlertDialogCancel disabled={isPending}>Annuler</AlertDialogCancel>
					<AlertDialogAction
						disabled={isPending}
						onClick={(event) => {
							event.preventDefault()
							onConfirm()
						}}
					>
						{isPending ? 'Confirmation…' : 'Confirmer quand même'}
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}

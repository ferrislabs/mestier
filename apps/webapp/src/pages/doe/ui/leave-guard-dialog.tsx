import { useBlocker } from '@tanstack/react-router'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { leavesEditor } from '#/pages/doe/lib/leave-guard'

export function LeaveGuardDialog({ active }: { active: boolean }) {
	const blocker = useBlocker({
		shouldBlockFn: leavesEditor,
		disabled: !active,
		enableBeforeUnload: true,
		withResolver: true,
	})
	return (
		<ConfirmDialog
			open={blocker.status === 'blocked'}
			onOpenChange={(open) => {
				if (!open) blocker.reset?.()
			}}
			title="Quitter sans enregistrer ?"
			description="Ce DOE a des modifications non enregistrées. Si tu quittes l’éditeur, elles seront perdues."
			confirmLabel="Quitter sans enregistrer"
			destructive
			onConfirm={() => blocker.proceed?.()}
		/>
	)
}

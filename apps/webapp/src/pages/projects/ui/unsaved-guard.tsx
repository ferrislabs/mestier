import { useBlocker } from '@tanstack/react-router'
import { useEffect } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useBeforeUnload } from '#/pages/projects/lib/use-before-unload'

export function UnsavedGuard({ dirty }: { dirty: boolean }) {
	useBeforeUnload(dirty)
	const blocker = useBlocker({
		shouldBlockFn: () => dirty,
		disabled: !dirty,
		enableBeforeUnload: false,
		withResolver: true,
	})
	return (
		<ConfirmDialog
			open={blocker.status === 'blocked'}
			onOpenChange={(open) => {
				if (!open) blocker.reset?.()
			}}
			title="Quitter sans enregistrer ?"
			description="Tes modifications non enregistrées seront perdues."
			confirmLabel="Quitter sans enregistrer"
			destructive
			onConfirm={() => blocker.proceed?.()}
		/>
	)
}

export function useReportDirty(
	dirty: boolean,
	onDirtyChange: (dirty: boolean) => void,
) {
	useEffect(() => {
		onDirtyChange(dirty)
		return () => onDirtyChange(false)
	}, [dirty, onDirtyChange])
}

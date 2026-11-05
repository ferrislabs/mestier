import { CircleCheck, LoaderCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { Collection } from '#/pages/automation/lib/collections'
import {
	type CollectionDiff,
	versionWarning,
} from '#/pages/automation/lib/openapi-diff'
import { CollectionDiffView } from '#/pages/automation/ui/collection-diff-view'
import { SpecSourcePicker } from '#/pages/automation/ui/spec-source-picker'

export interface UpdateCandidate {
	collection: Collection
	diff: CollectionDiff
}

export function UpdateCollectionDialog({
	collection,
	sourceUrl,
	loading,
	error,
	candidate,
	onLoadUrl,
	onLoadFile,
	onSourceChange,
	onResetCandidate,
	onClose,
	onConfirm,
}: {
	collection: Collection
	sourceUrl: string
	loading: boolean
	error: string | null
	candidate: UpdateCandidate | null
	onLoadUrl: (url: string) => void
	onLoadFile: (file: File) => void
	onSourceChange: () => void
	onResetCandidate: () => void
	onClose: () => void
	onConfirm: () => void
}) {
	const upToDate =
		candidate?.diff.identical && candidate.diff.versionChange === 'same'

	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DialogContent
				showCloseButton={false}
				className="flex max-h-[85vh] flex-col sm:max-w-2xl"
			>
				<DialogHeader>
					<DialogTitle>Mettre à jour « {collection.title} »</DialogTitle>
					<DialogDescription>
						Version installée : {collection.version}. La nouvelle spec est
						comparée à celle-ci avant toute modification.
					</DialogDescription>
				</DialogHeader>
				<div className="-mx-6 min-h-0 flex-1 overflow-y-auto px-6">
					{loading && !candidate ? (
						<p className="flex items-center gap-2 py-6 text-muted-foreground text-sm">
							<LoaderCircle className="size-4 animate-spin" aria-hidden />
							Récupération de la spec…
						</p>
					) : candidate ? (
						upToDate ? (
							<p className="flex items-center gap-2 py-4 text-sm">
								<CircleCheck
									className="size-4 text-(color:--auto-success-text)"
									aria-hidden
								/>
								La collection est à jour : la spec publiée est identique à la
								version {collection.version}.
							</p>
						) : (
							<CollectionDiffView
								diff={candidate.diff}
								warning={versionWarning(candidate.diff)}
							/>
						)
					) : (
						<SpecSourcePicker
							initialUrl={sourceUrl}
							loading={loading}
							submitLabel="Comparer"
							onLoadUrl={onLoadUrl}
							onLoadFile={onLoadFile}
							onSourceChange={onSourceChange}
						/>
					)}
					{error && (
						<p role="alert" className="mt-3 text-sm text-state-alert">
							{error}
						</p>
					)}
				</div>
				<DialogFooter>
					{candidate && (
						<Button
							variant="ghost"
							className="sm:mr-auto"
							onClick={onResetCandidate}
						>
							Comparer une autre source
						</Button>
					)}
					<Button variant="outline" onClick={onClose}>
						{upToDate ? 'Fermer' : 'Annuler'}
					</Button>
					{candidate && !upToDate && (
						<Button onClick={onConfirm}>
							Mettre à jour vers {candidate.collection.version}
						</Button>
					)}
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}

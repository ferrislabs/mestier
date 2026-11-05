import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { operationCount } from '#/pages/automation/lib/collection-format'
import type { Collection } from '#/pages/automation/lib/collections'
import { SpecSourcePicker } from '#/pages/automation/ui/spec-source-picker'

function CollectionPreview({ collection }: { collection: Collection }) {
	const tags = [
		...new Set(collection.operations.map((operation) => operation.tag)),
	]
	return (
		<dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 rounded-md border border-border bg-muted/40 p-4 text-sm">
			<dt className="text-muted-foreground">Titre</dt>
			<dd className="font-medium">{collection.title}</dd>
			<dt className="text-muted-foreground">Version</dt>
			<dd className="font-mono">{collection.version}</dd>
			<dt className="text-muted-foreground">URL de base</dt>
			<dd className="truncate font-mono">
				{collection.baseUrl || 'Non précisée'}
			</dd>
			<dt className="text-muted-foreground">Opérations</dt>
			<dd>
				{operationCount(collection.operations.length)}, regroupées en{' '}
				{tags.join(', ')}
			</dd>
		</dl>
	)
}

export function ImportCollectionDialog({
	loading,
	error,
	preview,
	onLoadUrl,
	onLoadFile,
	onSourceChange,
	onBack,
	onCancel,
	onConfirm,
}: {
	loading: boolean
	error: string | null
	preview: Collection | null
	onLoadUrl: (url: string) => void
	onLoadFile: (file: File) => void
	onSourceChange: () => void
	onBack: () => void
	onCancel: () => void
	onConfirm: () => void
}) {
	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onCancel()
			}}
		>
			<DialogContent showCloseButton={false} className="sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>Importer une collection</DialogTitle>
					<DialogDescription>
						Une spec OpenAPI 3, en JSON ou en YAML. Ses opérations deviennent
						des nœuds utilisables dans tous les workflows.
					</DialogDescription>
				</DialogHeader>
				{preview ? (
					<CollectionPreview collection={preview} />
				) : (
					<SpecSourcePicker
						loading={loading}
						submitLabel="Charger"
						onLoadUrl={onLoadUrl}
						onLoadFile={onLoadFile}
						onSourceChange={onSourceChange}
					/>
				)}
				{error && (
					<p role="alert" className="text-sm text-state-alert">
						{error}
					</p>
				)}
				<DialogFooter>
					<Button variant="outline" onClick={preview ? onBack : onCancel}>
						{preview ? 'Retour' : 'Annuler'}
					</Button>
					{preview && (
						<Button onClick={onConfirm}>Ajouter la collection</Button>
					)}
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}

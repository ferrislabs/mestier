import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { FilePreview } from '#/hooks/use-file-url'
import type { QuoteLineFormValues } from '../types'
import { QuoteLineEditor } from './quote-line-editor'

/** Shared with each `QuoteLineEditor` row so the CSS grid reads as one table —
 * see the prop's doc on `QuoteLineEditorProps` for why this isn't a real
 * `<table>`. The remove-button column stays fixed width in both variants. */
function gridTemplateColumns(vatEnabled: boolean): string {
	return vatEnabled
		? '1fr 100px 110px 80px 110px 40px'
		: '1fr 100px 110px 110px 40px'
}

interface QuoteLinesTableProps {
	lines: QuoteLineFormValues[]
	catalogItems: CatalogItem[]
	photosByLine: Record<string, FilePreview[]>
	isUploading?: boolean
	openLineId: string | null
	vatEnabled: boolean
	onOpenLineChange: (clientId: string, open: boolean) => void
	onLineChange: (clientId: string, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (clientId: string, catalogItemId: string) => void
	onRemoveLine: (clientId: string) => void
	onAddLine: () => void
	onUploadLinePhoto: (clientId: string, file: File) => void
	onRemoveLinePhoto: (clientId: string, key: string) => void
}

/**
 * The quote as a document reads as a table of lines; editing one needs room a
 * table cell doesn't have. This renders the header bar for that table and
 * folds each row into `QuoteLineEditor`, passing down the one grid template
 * both must share for the columns to actually line up.
 */
export function QuoteLinesTable({
	lines,
	catalogItems,
	photosByLine,
	isUploading,
	openLineId,
	vatEnabled,
	onOpenLineChange,
	onLineChange,
	onSelectCatalogItem,
	onRemoveLine,
	onAddLine,
	onUploadLinePhoto,
	onRemoveLinePhoto,
}: QuoteLinesTableProps) {
	const columns = gridTemplateColumns(vatEnabled)

	return (
		<div>
			<div
				className="grid items-center gap-3 border-b-2 border-foreground py-1.5 pr-2 pl-3 text-xs font-semibold tracking-wide text-muted-foreground uppercase"
				style={{ gridTemplateColumns: columns }}
			>
				<span>Détails</span>
				<span className="justify-self-end">Quantité</span>
				<span className="justify-self-end">Prix unitaire</span>
				{vatEnabled ? <span className="justify-self-end">TVA</span> : null}
				<span className="justify-self-end">Montant HT</span>
				<span />
			</div>

			<div>
				{lines.map((line, index) => (
					<QuoteLineEditor
						key={line.clientId}
						index={index}
						line={line}
						catalogItems={catalogItems}
						photos={photosByLine[line.clientId] ?? []}
						isOpen={openLineId === line.clientId}
						canRemove={lines.length > 1}
						isUploading={isUploading}
						vatEnabled={vatEnabled}
						gridTemplateColumns={columns}
						onOpenChange={(open) => onOpenLineChange(line.clientId, open)}
						onChange={(patch) => onLineChange(line.clientId, patch)}
						onSelectCatalogItem={(catalogItemId) =>
							onSelectCatalogItem(line.clientId, catalogItemId)
						}
						onRemove={() => onRemoveLine(line.clientId)}
						onUploadPhoto={(file) => onUploadLinePhoto(line.clientId, file)}
						onRemovePhoto={(key) => onRemoveLinePhoto(line.clientId, key)}
					/>
				))}
			</div>

			<div className="border-t p-2">
				<Button type="button" variant="ghost" size="sm" onClick={onAddLine}>
					<Plus />
					Ajouter une ligne
				</Button>
			</div>
		</div>
	)
}

import { ExternalLink, FileText, Loader2, Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'

export const DATASHEET_ACCEPT =
	'application/pdf,image/jpeg,image/png,image/webp'

export interface TechnicalSheetValues {
	supplierId: string
	manufacturer: string
	technicalSpecs: string
	datasheetKeys: string[]
}

export interface SupplierOption {
	id: string
	name: string
}

export interface DatasheetFile {
	key: string
	name: string
	type: string
	url: string | undefined
}

export interface ProductTechnicalSheetProps {
	values: TechnicalSheetValues
	suppliers: SupplierOption[]
	datasheets: DatasheetFile[]
	isUploading?: boolean
	canCreateSupplier: boolean
	onChange: (patch: Partial<TechnicalSheetValues>) => void
	onUpload: (file: File) => void
	onRemove: (key: string) => void
	onCreateSupplier: () => void
}

export function ProductTechnicalSheet({
	values,
	suppliers,
	datasheets,
	isUploading,
	canCreateSupplier,
	onChange,
	onUpload,
	onRemove,
	onCreateSupplier,
}: ProductTechnicalSheetProps) {
	return (
		<fieldset className="space-y-4 rounded-lg border p-4">
			<legend className="px-1 text-sm font-semibold">Fiche technique</legend>

			<div className="flex flex-col gap-2">
				<div className="flex items-center justify-between gap-2">
					<Label htmlFor="product-supplier">Fournisseur</Label>
					{canCreateSupplier ? (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={onCreateSupplier}
						>
							<Plus />
							Nouveau fournisseur
						</Button>
					) : null}
				</div>
				<select
					id="product-supplier"
					value={values.supplierId}
					onChange={(event) => onChange({ supplierId: event.target.value })}
					className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
				>
					<option value="">Aucun fournisseur</option>
					{suppliers.map((supplier) => (
						<option key={supplier.id} value={supplier.id}>
							{supplier.name}
						</option>
					))}
				</select>
			</div>

			<div className="flex flex-col gap-2">
				<Label htmlFor="product-manufacturer">Fabricant / pépinière</Label>
				<Input
					id="product-manufacturer"
					value={values.manufacturer}
					onChange={(event) => onChange({ manufacturer: event.target.value })}
					placeholder="Optionnel"
				/>
			</div>

			<div className="flex flex-col gap-2">
				<Label htmlFor="product-specs">Caractéristiques techniques</Label>
				<Textarea
					id="product-specs"
					value={values.technicalSpecs}
					onChange={(event) => onChange({ technicalSpecs: event.target.value })}
					placeholder="Dimensions, référence, mise en œuvre… Une caractéristique par ligne."
					rows={5}
				/>
			</div>

			<div className="flex flex-col gap-2">
				<Label>Documents du fournisseur</Label>
				{datasheets.length > 0 ? (
					<ul className="divide-y rounded-md border">
						{datasheets.map((file) => (
							<li
								key={file.key}
								className="flex items-center gap-2 px-3 py-2 text-sm"
							>
								<FileText className="size-4 shrink-0 text-muted-foreground" />
								<span className="min-w-0 flex-1 truncate">{file.name}</span>
								<span className="shrink-0 text-xs text-muted-foreground">
									{file.type}
								</span>
								{file.url ? (
									<a
										href={file.url}
										target="_blank"
										rel="noreferrer"
										className="shrink-0 text-primary"
									>
										<ExternalLink className="size-4" />
										<span className="sr-only">Ouvrir {file.name}</span>
									</a>
								) : null}
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									onClick={() => onRemove(file.key)}
								>
									<X className="size-4" />
									<span className="sr-only">Retirer {file.name}</span>
								</Button>
							</li>
						))}
					</ul>
				) : null}
				<label className="flex w-fit cursor-pointer items-center gap-2 rounded-md border border-dashed px-3 py-2 text-sm text-muted-foreground transition-colors hover:border-primary hover:text-primary">
					{isUploading ? (
						<Loader2 className="size-4 animate-spin" />
					) : (
						<Plus className="size-4" />
					)}
					Ajouter un document (PDF ou image)
					<input
						type="file"
						accept={DATASHEET_ACCEPT}
						className="sr-only"
						disabled={isUploading}
						onChange={(event) => {
							const file = event.target.files?.[0]
							if (file) onUpload(file)
							event.target.value = ''
						}}
					/>
				</label>
			</div>
		</fieldset>
	)
}

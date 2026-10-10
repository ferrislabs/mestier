import { Loader2, X } from 'lucide-react'
import { FileUploadButton } from '#/components/file-upload-button'
import { TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Textarea } from '#/components/ui/textarea'
import {
	canSubmitSupply,
	type SupplyFormValues,
} from '#/pages/doe/lib/supply-form'
import { MAKER_LABEL_TEXT, type MakerLabel } from '#/pages/doe/types'

interface DoeSupplySheetProps {
	open: boolean
	isNew: boolean
	values: SupplyFormValues
	makerLabel: MakerLabel
	fileUrls: Record<string, string | undefined>
	isUploadingPhotos: boolean
	isUploadingDatasheets: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<SupplyFormValues>) => void
	onUploadPhotos: (files: File[]) => void
	onUploadDatasheets: (files: File[]) => void
	onSubmit: () => void
}

export function DoeSupplySheet({
	open,
	isNew,
	values,
	makerLabel,
	fileUrls,
	isUploadingPhotos,
	isUploadingDatasheets,
	error,
	onOpenChange,
	onChange,
	onUploadPhotos,
	onUploadDatasheets,
	onSubmit,
}: DoeSupplySheetProps) {
	const makerText = MAKER_LABEL_TEXT[makerLabel]

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full overflow-y-auto sm:max-w-xl">
				<SheetHeader>
					<SheetTitle>
						{isNew ? 'Nouvelle fourniture' : 'Modifier la fourniture'}
					</SheetTitle>
					<SheetDescription>
						Modifier ici ne change que ce DOE, pas le catalogue.
					</SheetDescription>
				</SheetHeader>

				<div className="flex flex-1 flex-col gap-6 px-4">
					<fieldset className="flex flex-col gap-3">
						<legend className="mb-2 text-sm font-semibold">
							Propre à ce chantier
						</legend>
						<TextField
							label="Titre dans le DOE"
							value={values.title}
							onChange={(title) => onChange({ title })}
						/>
						<TextField
							label="Correspondance DPGF"
							value={values.dpgfRef}
							onChange={(dpgfRef) => onChange({ dpgfRef })}
						/>
						<TextField
							label="Destination"
							value={values.destination}
							onChange={(destination) => onChange({ destination })}
						/>
					</fieldset>

					<fieldset className="flex flex-col gap-3">
						<legend className="mb-2 text-sm font-semibold">Fiche</legend>
						<TextField
							label="Désignation"
							required
							value={values.designation}
							onChange={(designation) => onChange({ designation })}
						/>
						<TextField
							label="Nom du fournisseur"
							value={values.supplierName}
							onChange={(supplierName) => onChange({ supplierName })}
						/>
						<div className="flex flex-col gap-2">
							<Label htmlFor="supply-supplier-address">
								Adresse du fournisseur
							</Label>
							<Textarea
								id="supply-supplier-address"
								rows={2}
								value={values.supplierAddress}
								onChange={(event) =>
									onChange({ supplierAddress: event.target.value })
								}
							/>
						</div>
						<TextField
							label="Téléphone du fournisseur"
							value={values.supplierPhone}
							onChange={(supplierPhone) => onChange({ supplierPhone })}
						/>
						<TextField
							label="Email du fournisseur"
							type="email"
							value={values.supplierEmail}
							onChange={(supplierEmail) => onChange({ supplierEmail })}
						/>
						<TextField
							label={makerText}
							value={values.maker}
							onChange={(maker) => onChange({ maker })}
						/>
						<div className="flex flex-col gap-2">
							<Label htmlFor="supply-specs">Caractéristiques techniques</Label>
							<Textarea
								id="supply-specs"
								rows={4}
								value={values.specs}
								onChange={(event) => onChange({ specs: event.target.value })}
							/>
						</div>
					</fieldset>

					<fieldset className="flex flex-col gap-3">
						<legend className="mb-2 text-sm font-semibold">
							Pièces jointes
						</legend>
						<div className="flex items-center justify-between">
							<Label>Photos</Label>
							<FileUploadButton
								accept="image/*"
								multiple
								isUploading={isUploadingPhotos}
								onFiles={onUploadPhotos}
							>
								Ajouter des photos
							</FileUploadButton>
						</div>
						{values.photos.length === 0 ? (
							<p className="text-xs text-muted-foreground">Aucune photo.</p>
						) : (
							<ul className="grid grid-cols-2 gap-3">
								{values.photos.map((photo, index) => {
									const url = fileUrls[photo.key]
									return (
										<li key={photo.key} className="space-y-1">
											<div className="relative">
												{url ? (
													<img
														src={url}
														alt={photo.caption ?? `Photo ${index + 1}`}
														className="aspect-video w-full rounded border object-cover"
													/>
												) : (
													<div className="aspect-video w-full rounded border bg-muted" />
												)}
												<Button
													type="button"
													variant="secondary"
													size="icon-xs"
													className="absolute top-1 right-1"
													onClick={() =>
														onChange({
															photos: values.photos.filter(
																(_, position) => position !== index,
															),
														})
													}
												>
													<X />
													<span className="sr-only">
														Retirer la photo {index + 1}
													</span>
												</Button>
											</div>
											<input
												className="w-full rounded-md border bg-card px-2 py-1 text-xs"
												value={photo.caption ?? ''}
												placeholder="Légende"
												aria-label={`Légende de la photo ${index + 1}`}
												onChange={(event) =>
													onChange({
														photos: values.photos.map((entry, position) =>
															position === index
																? { ...entry, caption: event.target.value }
																: entry,
														),
													})
												}
											/>
										</li>
									)
								})}
							</ul>
						)}

						<div className="mt-2 flex items-center justify-between">
							<Label>Documents du fournisseur</Label>
							<FileUploadButton
								accept="application/pdf,image/*"
								multiple
								isUploading={isUploadingDatasheets}
								onFiles={onUploadDatasheets}
							>
								Ajouter des documents
							</FileUploadButton>
						</div>
						{values.datasheetKeys.length === 0 ? (
							<p className="text-xs text-muted-foreground">Aucun document.</p>
						) : (
							<ul className="space-y-1">
								{values.datasheetKeys.map((key, index) => (
									<li
										key={key}
										className="flex items-center justify-between gap-2 rounded border px-3 py-1.5 text-sm"
									>
										{fileUrls[key] ? (
											<a
												href={fileUrls[key]}
												target="_blank"
												rel="noreferrer"
												className="truncate underline"
											>
												Document {index + 1}
											</a>
										) : (
											<span className="truncate">Document {index + 1}</span>
										)}
										<Button
											type="button"
											variant="ghost"
											size="icon-xs"
											onClick={() =>
												onChange({
													datasheetKeys: values.datasheetKeys.filter(
														(_, position) => position !== index,
													),
												})
											}
										>
											<X />
											<span className="sr-only">
												Retirer le document {index + 1}
											</span>
										</Button>
									</li>
								))}
							</ul>
						)}
					</fieldset>
					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>

				<SheetFooter className="flex-row justify-end border-t">
					<Button
						type="button"
						variant="outline"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button
						type="button"
						disabled={
							!canSubmitSupply(values) ||
							isUploadingPhotos ||
							isUploadingDatasheets
						}
						onClick={onSubmit}
					>
						{isUploadingPhotos || isUploadingDatasheets ? (
							<Loader2 className="animate-spin" />
						) : null}
						{isNew ? 'Ajouter' : 'Appliquer'}
					</Button>
				</SheetFooter>
			</SheetContent>
		</Sheet>
	)
}

import { FileText, ImageIcon, Loader2, X } from 'lucide-react'
import { type ReactNode, useId } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import {
	canSubmitSupply,
	type SupplyFormValues,
} from '#/pages/doe/lib/supply-form'
import { MAKER_LABEL_TEXT, type MakerLabel } from '#/pages/doe/types'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import { IconButton } from '#/pages/doe/ui/icon-button'
import { SideSheet } from '#/pages/doe/ui/side-sheet'

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

function FormSection({
	label,
	hint,
	children,
}: {
	label: string
	hint?: string
	children: ReactNode
}) {
	return (
		<section aria-label={label} className="grid gap-3">
			<div>
				<h3 className="font-semibold text-sm">{label}</h3>
				{hint && <p className="text-[12px] text-muted-foreground">{hint}</p>}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">{children}</div>
		</section>
	)
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
	const baseId = useId()
	const id = (field: string) => `${baseId}-${field}`
	const makerText = MAKER_LABEL_TEXT[makerLabel]
	const busy = isUploadingPhotos || isUploadingDatasheets
	const canSubmit = canSubmitSupply(values) && !busy

	return (
		<SideSheet
			open={open}
			onOpenChange={onOpenChange}
			title={isNew ? 'Nouvelle fourniture' : 'Modifier la fourniture'}
			description="Modifier ici ne change que ce DOE, pas le catalogue."
		>
			<form
				noValidate
				onSubmit={(event) => {
					event.preventDefault()
					if (canSubmit) onSubmit()
				}}
				className="flex min-h-0 flex-1 flex-col"
			>
				<div className="grid min-h-0 flex-1 content-start gap-6 overflow-y-auto px-6 py-5">
					<FormSection
						label="Propre à ce chantier"
						hint="Ces informations ne concernent que ce DOE."
					>
						<FormField
							id={id('title')}
							label="Titre dans le DOE"
							hint="Remplace la désignation dans le DOE. Laisse vide pour la garder."
							className="sm:col-span-2"
						>
							<Input
								id={id('title')}
								value={values.title}
								autoComplete="off"
								onChange={(event) => onChange({ title: event.target.value })}
							/>
						</FormField>
						<FormField id={id('dpgf')} label="Correspondance DPGF">
							<Input
								id={id('dpgf')}
								value={values.dpgfRef}
								autoComplete="off"
								onChange={(event) => onChange({ dpgfRef: event.target.value })}
							/>
						</FormField>
						<FormField id={id('destination')} label="Destination">
							<Input
								id={id('destination')}
								value={values.destination}
								autoComplete="off"
								onChange={(event) =>
									onChange({ destination: event.target.value })
								}
							/>
						</FormField>
					</FormSection>

					<FormSection label="Fiche">
						<FormField
							id={id('designation')}
							label="Désignation"
							required
							className="sm:col-span-2"
						>
							<Input
								id={id('designation')}
								value={values.designation}
								autoComplete="off"
								aria-required
								onChange={(event) =>
									onChange({ designation: event.target.value })
								}
							/>
						</FormField>
						<FormField
							id={id('supplier-name')}
							label="Nom du fournisseur"
							hint="Les coordonnées ne sont conservées qu’avec un nom."
							className="sm:col-span-2"
						>
							<Input
								id={id('supplier-name')}
								value={values.supplierName}
								autoComplete="off"
								onChange={(event) =>
									onChange({ supplierName: event.target.value })
								}
							/>
						</FormField>
						<FormField
							id={id('supplier-address')}
							label="Adresse du fournisseur"
							className="sm:col-span-2"
						>
							<Textarea
								id={id('supplier-address')}
								rows={2}
								value={values.supplierAddress}
								onChange={(event) =>
									onChange({ supplierAddress: event.target.value })
								}
							/>
						</FormField>
						<FormField
							id={id('supplier-phone')}
							label="Téléphone du fournisseur"
						>
							<Input
								id={id('supplier-phone')}
								type="tel"
								value={values.supplierPhone}
								autoComplete="off"
								onChange={(event) =>
									onChange({ supplierPhone: event.target.value })
								}
							/>
						</FormField>
						<FormField id={id('supplier-email')} label="Email du fournisseur">
							<Input
								id={id('supplier-email')}
								type="email"
								value={values.supplierEmail}
								autoComplete="off"
								onChange={(event) =>
									onChange({ supplierEmail: event.target.value })
								}
							/>
						</FormField>
						<FormField
							id={id('maker')}
							label={makerText}
							className="sm:col-span-2"
						>
							<Input
								id={id('maker')}
								value={values.maker}
								autoComplete="off"
								onChange={(event) => onChange({ maker: event.target.value })}
							/>
						</FormField>
						<FormField
							id={id('specs')}
							label="Caractéristiques techniques"
							className="sm:col-span-2"
						>
							<Textarea
								id={id('specs')}
								rows={4}
								value={values.specs}
								onChange={(event) => onChange({ specs: event.target.value })}
							/>
						</FormField>

						<div className="grid gap-2 sm:col-span-2">
							<h4 className="font-medium text-sm">Photos</h4>
							<FileDropZone
								accept="image/*"
								label="Dépose des photos ici"
								hint="Ou clique pour choisir des images"
								compact
								isUploading={isUploadingPhotos}
								onFiles={onUploadPhotos}
							/>
							{values.photos.length === 0 ? (
								<p className="text-muted-foreground text-xs">Aucune photo.</p>
							) : (
								<ul className="grid grid-cols-2 gap-3">
									{values.photos.map((photo, index) => {
										const url = fileUrls[photo.key]
										const position = index + 1
										return (
											<li key={photo.key} className="grid content-start gap-2">
												{url ? (
													<img
														src={url}
														alt={photo.caption?.trim() || `Photo ${position}`}
														className="aspect-[4/3] w-full rounded-md border object-cover"
													/>
												) : (
													<div
														aria-hidden
														className="flex aspect-[4/3] w-full items-center justify-center rounded-md border bg-muted text-muted-foreground"
													>
														<ImageIcon className="size-6" />
													</div>
												)}
												<Input
													value={photo.caption ?? ''}
													placeholder="Légende"
													aria-label={`Légende de la photo ${position}`}
													autoComplete="off"
													className="h-8"
													onChange={(event) =>
														onChange({
															photos: values.photos.map((entry, at) =>
																at === index
																	? { ...entry, caption: event.target.value }
																	: entry,
															),
														})
													}
												/>
												<div className="flex items-center gap-0.5">
													<span className="flex-1" />
													<IconButton
														label={`Retirer la photo ${position}`}
														icon={X}
														onClick={() =>
															onChange({
																photos: values.photos.filter(
																	(_, at) => at !== index,
																),
															})
														}
													/>
												</div>
											</li>
										)
									})}
								</ul>
							)}
						</div>

						<div className="grid gap-2 sm:col-span-2">
							<h4 className="font-medium text-sm">Fiches techniques</h4>
							<FileDropZone
								accept="application/pdf,image/*"
								label="Dépose des fiches techniques ici"
								hint="Ou clique pour choisir des PDF ou des images"
								compact
								isUploading={isUploadingDatasheets}
								onFiles={onUploadDatasheets}
							/>
							{values.datasheetKeys.length === 0 ? (
								<p className="text-muted-foreground text-xs">
									Aucune fiche technique.
								</p>
							) : (
								<ul className="divide-y rounded-md border">
									{values.datasheetKeys.map((key, index) => (
										<li key={key} className="flex items-center gap-3 px-3 py-2">
											<FileText
												aria-hidden
												className="size-4 shrink-0 text-muted-foreground"
											/>
											<div className="min-w-0 flex-1">
												{fileUrls[key] ? (
													<a
														href={fileUrls[key]}
														target="_blank"
														rel="noreferrer"
														className="block truncate text-sm underline"
													>
														Document {index + 1}
													</a>
												) : (
													<p className="truncate text-sm">
														Document {index + 1}
													</p>
												)}
											</div>
											<IconButton
												label={`Retirer le document ${index + 1}`}
												icon={X}
												onClick={() =>
													onChange({
														datasheetKeys: values.datasheetKeys.filter(
															(_, at) => at !== index,
														),
													})
												}
											/>
										</li>
									))}
								</ul>
							)}
						</div>
					</FormSection>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
				</div>
				<div className="flex justify-end gap-2 border-t px-6 py-4">
					<Button
						type="button"
						variant="outline"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button type="submit" disabled={!canSubmit}>
						{busy ? <Loader2 className="animate-spin" /> : null}
						{isNew ? 'Ajouter' : 'Appliquer'}
					</Button>
				</div>
			</form>
		</SideSheet>
	)
}

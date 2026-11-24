import { Images, Trash2, X } from 'lucide-react'
import { useId } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'
import { type DafFormValues, MAX_ATTACHMENTS } from '#/pages/daf/lib/daf-form'
import { type DafStatus, STATUS_OPTIONS } from '#/pages/daf/lib/daf-status'
import { EditorCard } from '#/pages/doe/ui/editor-card'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'

export interface DafEditorFormProps {
	values: DafFormValues
	dossierOwner: string
	dossierManager: string
	photoUrl: string | undefined
	isUploadingPhoto: boolean
	isUploadingAttachments: boolean
	libraryBlockedReason: string | null
	canEdit: boolean
	onChange: (patch: Partial<DafFormValues>) => void
	onUploadPhoto: (files: File[]) => void
	onPickPhotoFromLibrary: () => void
	onUploadAttachments: (files: File[]) => void
	onAttachmentNameChange: (index: number, name: string) => void
	onRemoveAttachment: (index: number) => void
}

function isDafStatus(value: string): value is DafStatus {
	return STATUS_OPTIONS.some((option) => option.value === value)
}

export function DafEditorForm({
	values,
	dossierOwner,
	dossierManager,
	photoUrl,
	isUploadingPhoto,
	isUploadingAttachments,
	libraryBlockedReason,
	canEdit,
	onChange,
	onUploadPhoto,
	onPickPhotoFromLibrary,
	onUploadAttachments,
	onAttachmentNameChange,
	onRemoveAttachment,
}: DafEditorFormProps) {
	const base = useId()
	const id = (field: string) => `${base}-${field}`
	const text =
		(key: keyof DafFormValues) => (event: { target: { value: string } }) =>
			onChange({ [key]: event.target.value })

	return (
		<div className="@container grid gap-6">
			<EditorCard title="Identification">
				<div className="grid gap-4 p-5 @xl:grid-cols-2">
					<FormField id={id('number')} label="Numéro" required>
						<Input
							id={id('number')}
							type="number"
							min={1}
							max={999}
							value={values.number}
							disabled={!canEdit}
							onChange={text('number')}
						/>
					</FormField>
					<FormField id={id('price-ref')} label="Correspondance (N° prix)">
						<Input
							id={id('price-ref')}
							value={values.priceRef}
							disabled={!canEdit}
							onChange={text('priceRef')}
						/>
					</FormField>
					<FormField
						id={id('designation')}
						label="Désignation"
						required
						className="@xl:col-span-2"
					>
						<Input
							id={id('designation')}
							value={values.designation}
							maxLength={200}
							disabled={!canEdit}
							onChange={text('designation')}
						/>
					</FormField>
				</div>
			</EditorCard>

			<EditorCard title="Fournisseur">
				<div className="grid gap-4 p-5 @xl:grid-cols-2">
					<FormField id={id('supplier-name')} label="Nom">
						<Input
							id={id('supplier-name')}
							value={values.supplierName}
							disabled={!canEdit}
							onChange={text('supplierName')}
						/>
					</FormField>
					<FormField id={id('supplier-address')} label="Adresse">
						<Input
							id={id('supplier-address')}
							value={values.supplierAddress}
							disabled={!canEdit}
							onChange={text('supplierAddress')}
						/>
					</FormField>
					<FormField id={id('supplier-phone')} label="Téléphone">
						<Input
							id={id('supplier-phone')}
							value={values.supplierPhone}
							disabled={!canEdit}
							onChange={text('supplierPhone')}
						/>
					</FormField>
					<FormField id={id('supplier-email')} label="E-mail">
						<Input
							id={id('supplier-email')}
							type="email"
							value={values.supplierEmail}
							disabled={!canEdit}
							onChange={text('supplierEmail')}
						/>
					</FormField>
				</div>
			</EditorCard>

			<EditorCard title="Description">
				<div className="grid gap-4 p-5">
					<FormField id={id('installer')} label="Assemblage et pose">
						<Textarea
							id={id('installer')}
							rows={3}
							value={values.installer}
							disabled={!canEdit}
							onChange={text('installer')}
						/>
					</FormField>
					<FormField id={id('specs')} label="Caractéristiques techniques">
						<Textarea
							id={id('specs')}
							rows={5}
							value={values.specs}
							disabled={!canEdit}
							onChange={text('specs')}
						/>
					</FormField>
					<FormField id={id('observations')} label="Observations">
						<Textarea
							id={id('observations')}
							rows={3}
							value={values.observations}
							disabled={!canEdit}
							onChange={text('observations')}
						/>
					</FormField>
				</div>
			</EditorCard>

			<EditorCard
				title="Image"
				description="Une seule image figure sur la fiche."
			>
				<div className="grid gap-4 p-5">
					{values.photoKey ? (
						<div className="flex flex-wrap items-start gap-4">
							{photoUrl ? (
								<img
									src={photoUrl}
									alt="Aperçu de la DAF"
									className="max-h-48 rounded-md border object-contain"
								/>
							) : (
								<span className="text-muted-foreground text-sm">
									Image envoyée
								</span>
							)}
							{canEdit ? (
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={() => onChange({ photoKey: '' })}
								>
									<Trash2 aria-hidden />
									Retirer l’image
								</Button>
							) : null}
						</div>
					) : null}
					{canEdit ? (
						<>
							<FileDropZone
								accept="image/jpeg,image/png,image/webp"
								label="Déposer une image"
								hint="JPEG, PNG ou WebP"
								isUploading={isUploadingPhoto}
								compact
								onFiles={(files) => onUploadPhoto(files.slice(0, 1))}
							/>
							<div className="grid justify-items-start gap-1">
								<Button
									type="button"
									variant="outline"
									size="sm"
									disabled={libraryBlockedReason !== null}
									onClick={onPickPhotoFromLibrary}
								>
									<Images aria-hidden />
									Choisir dans la photothèque
								</Button>
								{libraryBlockedReason ? (
									<p className="text-[12px] text-muted-foreground">
										{libraryBlockedReason}
									</p>
								) : null}
							</div>
						</>
					) : null}
				</div>
			</EditorCard>

			<EditorCard
				title="Pièces jointes"
				description={`Fiches techniques et documents, ${MAX_ATTACHMENTS} au plus.`}
			>
				<div className="grid gap-4 p-5">
					{values.attachments.length > 0 ? (
						<ul className="grid gap-2">
							{values.attachments.map((attachment, index) => (
								<li key={attachment.key} className="flex items-center gap-2">
									<Input
										aria-label={`Nom de la pièce jointe ${index + 1}`}
										value={attachment.name}
										disabled={!canEdit}
										onChange={(event) =>
											onAttachmentNameChange(index, event.target.value)
										}
									/>
									{canEdit ? (
										<Button
											type="button"
											variant="ghost"
											size="icon"
											aria-label={`Retirer la pièce jointe ${index + 1}`}
											onClick={() => onRemoveAttachment(index)}
										>
											<X aria-hidden />
										</Button>
									) : null}
								</li>
							))}
						</ul>
					) : null}
					{canEdit ? (
						<FileDropZone
							accept="application/pdf,image/jpeg,image/png,image/webp"
							label="Déposer des fichiers"
							hint="PDF ou images"
							isUploading={isUploadingAttachments}
							disabled={values.attachments.length >= MAX_ATTACHMENTS}
							compact
							onFiles={onUploadAttachments}
						/>
					) : null}
				</div>
			</EditorCard>

			<EditorCard
				title="Diffusion"
				description="Laisser vide pour reprendre la valeur du dossier."
			>
				<div className="grid gap-4 p-5 @xl:grid-cols-2">
					<FormField id={id('owner')} label="Maîtrise d’ouvrage">
						<Input
							id={id('owner')}
							value={values.projectOwner}
							placeholder={dossierOwner}
							disabled={!canEdit}
							onChange={text('projectOwner')}
						/>
					</FormField>
					<FormField id={id('manager')} label="Maîtrise d’œuvre">
						<Input
							id={id('manager')}
							value={values.projectManager}
							placeholder={dossierManager}
							disabled={!canEdit}
							onChange={text('projectManager')}
						/>
					</FormField>
				</div>
			</EditorCard>

			<EditorCard title="Suivi">
				<div className="grid gap-4 p-5 @xl:grid-cols-2">
					<FormField id={id('status')} label="Statut">
						<Select
							value={values.status}
							disabled={!canEdit}
							onValueChange={(value) => {
								if (isDafStatus(value)) onChange({ status: value })
							}}
						>
							<SelectTrigger id={id('status')} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{STATUS_OPTIONS.map((option) => (
									<SelectItem key={option.value} value={option.value}>
										{option.label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<FormField id={id('issued-on')} label="Date de diffusion" required>
						<Input
							id={id('issued-on')}
							type="date"
							value={values.issuedOn}
							disabled={!canEdit}
							onChange={text('issuedOn')}
						/>
					</FormField>
					<FormField id={id('sent-on')} label="Envoyé le">
						<Input
							id={id('sent-on')}
							type="date"
							value={values.sentOn}
							disabled={!canEdit}
							onChange={text('sentOn')}
						/>
					</FormField>
					<FormField id={id('returned-on')} label="Retour le">
						<Input
							id={id('returned-on')}
							type="date"
							value={values.returnedOn}
							disabled={!canEdit}
							onChange={text('returnedOn')}
						/>
					</FormField>
					<FormField
						id={id('review-note')}
						label="Observation de la MOE"
						className="@xl:col-span-2"
					>
						<Textarea
							id={id('review-note')}
							rows={3}
							value={values.reviewNote}
							disabled={!canEdit}
							onChange={text('reviewNote')}
						/>
					</FormField>
				</div>
			</EditorCard>
		</div>
	)
}

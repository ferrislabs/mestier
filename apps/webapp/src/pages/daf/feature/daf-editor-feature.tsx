import { useForm, useStore } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useUploadFile } from '#/hooks/use-customers'
import { type Daf, useDeleteDaf, useUpdateDaf } from '#/hooks/use-daf'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import { useFileUrls } from '#/hooks/use-file-url'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { DafPhotoPickerFeature } from '#/pages/daf/feature/daf-photo-picker-feature'
import {
	buildUpdateBody,
	canSave,
	type DafFormValues,
	DEFAULT_ATTACHMENT_NAME,
	dafToForm,
	formErrors,
	MAX_ATTACHMENTS,
} from '#/pages/daf/lib/daf-form'
import { dafNumberLabel } from '#/pages/daf/lib/daf-status'
import { DafDeleteDialog } from '#/pages/daf/ui/daf-delete-dialog'
import { DafEditorUI } from '#/pages/daf/ui/daf-editor-ui'
import { LeaveGuardDialog } from '#/pages/doe/ui/leave-guard-dialog'

function fileName(file: File): string {
	return file.name.replace(/\.[^.]+$/, '') || DEFAULT_ATTACHMENT_NAME
}

export function DafEditorFeature({
	daf,
	dossier,
	organizationId,
	organizationSlug,
	projectId,
	projectName,
}: {
	daf: Daf
	dossier: Schemas.DafDossierResponse
	organizationId: string
	organizationSlug: string
	projectId: string
	projectName: string
}) {
	const initial = useMemo(() => dafToForm(daf), [daf])
	const updateDaf = useUpdateDaf()
	const deleteDaf = useDeleteDaf()
	const uploadFile = useUploadFile()
	const navigate = useNavigate()
	const canEdit = useHasPermission('MANAGE_PLANNING')
	const [token, setToken] = useState(daf.updated_at)
	const [notice, setNotice] = useState<string | null>(null)
	const [uploadError, setUploadError] = useState<string | null>(null)
	const [uploading, setUploading] = useState<'photo' | 'attachments' | null>(
		null,
	)
	const [libraryOpen, setLibraryOpen] = useState(false)
	const [deleteOpen, setDeleteOpen] = useState(false)
	const [deleted, setDeleted] = useState(false)

	const form = useForm({ defaultValues: initial })
	const values = useStore(form.store, (state) => state.values)
	const dirty = useDirtyBaseline(initial, values)

	useEffect(() => {
		setToken(daf.updated_at)
	}, [daf.updated_at])

	useEffect(() => {
		if (!notice) return
		const timer = setTimeout(() => setNotice(null), 4000)
		return () => clearTimeout(timer)
	}, [notice])

	useEffect(() => {
		if (!deleted) return
		void navigate({
			to: buildOrgPath(organizationSlug, '/projet/$projectId/daf'),
			params: { projectId },
		})
	}, [deleted, navigate, organizationSlug, projectId])

	const keys = useMemo(
		() => (values.photoKey ? [values.photoKey] : []),
		[values.photoKey],
	)
	const previews = useFileUrls(keys)

	const change = (patch: Partial<DafFormValues>) => {
		for (const [key, value] of Object.entries(patch)) {
			form.setFieldValue(key as keyof DafFormValues, value as never)
		}
	}

	const adopt = (next: Daf) => {
		const nextValues = dafToForm(next)
		form.reset(nextValues)
		dirty.commit(nextValues)
		setToken(next.updated_at)
	}

	const upload = async (files: File[]) => {
		setUploadError(null)
		const uploaded: Schemas.DafAttachment[] = []
		try {
			for (const file of files) {
				const response = await uploadFile.mutateAsync(file)
				uploaded.push({ key: response.data.key, name: fileName(file) })
			}
		} catch (error) {
			setUploadError(mutationErrorMessage(error))
		}
		return uploaded
	}

	const save = async () => {
		setNotice(null)
		const response = await updateDaf.mutateAsync({
			path: { organization_id: organizationId, daf_id: daf.id },
			body: buildUpdateBody(values, token),
		})
		adopt(response.data)
		setNotice('Enregistré')
	}

	return (
		<>
			<DafEditorUI
				organizationSlug={organizationSlug}
				projectId={projectId}
				projectName={projectName}
				number={Number(values.number) || daf.number}
				designation={values.designation}
				status={values.status}
				isDirty={dirty.isDirty}
				isSaving={updateDaf.isPending}
				canSave={canSave(values)}
				notice={notice}
				errors={formErrors(values)}
				error={mutationErrorMessage(updateDaf.error) ?? uploadError}
				form={{
					values,
					dossierOwner: dossier.project_owner ?? '',
					dossierManager: dossier.project_manager ?? '',
					photoUrl: previews[0]?.url,
					isUploadingPhoto: uploading === 'photo',
					isUploadingAttachments: uploading === 'attachments',
					libraryBlockedReason: dirty.isDirty
						? 'Enregistre tes modifications avant de choisir dans la photothèque.'
						: null,
					canEdit,
					onChange: change,
					onUploadPhoto: async (files) => {
						setUploading('photo')
						const [uploaded] = await upload(files)
						setUploading(null)
						if (uploaded) change({ photoKey: uploaded.key })
					},
					onPickPhotoFromLibrary: () => setLibraryOpen(true),
					onUploadAttachments: async (files) => {
						setUploading('attachments')
						const room = MAX_ATTACHMENTS - values.attachments.length
						const uploaded = await upload(files.slice(0, Math.max(0, room)))
						setUploading(null)
						change({ attachments: [...values.attachments, ...uploaded] })
					},
					onAttachmentNameChange: (index, name) =>
						change({
							attachments: values.attachments.map((attachment, position) =>
								position === index ? { ...attachment, name } : attachment,
							),
						}),
					onRemoveAttachment: (index) =>
						change({
							attachments: values.attachments.filter(
								(_, position) => position !== index,
							),
						}),
				}}
				onSave={() => {
					void save().catch(() => {})
				}}
				onPreview={() => {
					void navigate({
						to: buildOrgPath(
							organizationSlug,
							'/projet/$projectId/daf/$dafId/preview',
						),
						params: { projectId, dafId: daf.id },
					})
				}}
				onDelete={() => setDeleteOpen(true)}
			/>

			<LeaveGuardDialog
				active={dirty.isDirty && !deleted}
				description="Cette DAF a des modifications non enregistrées. Si tu quittes l’éditeur, elles seront perdues."
			/>

			<DafPhotoPickerFeature
				open={libraryOpen}
				organizationId={organizationId}
				dafId={daf.id}
				onPlaced={adopt}
				onClose={() => setLibraryOpen(false)}
			/>

			<DafDeleteDialog
				open={deleteOpen}
				label={dafNumberLabel(daf.number)}
				isPending={deleteDaf.isPending}
				error={mutationErrorMessage(deleteDaf.error)}
				onOpenChange={(next) => {
					if (!next) deleteDaf.reset()
					setDeleteOpen(next)
				}}
				onConfirm={() => {
					void deleteDaf
						.mutateAsync({
							path: { organization_id: organizationId, daf_id: daf.id },
						})
						.then(() => {
							setDeleteOpen(false)
							setDeleted(true)
						})
						.catch(() => {})
				}}
			/>
		</>
	)
}

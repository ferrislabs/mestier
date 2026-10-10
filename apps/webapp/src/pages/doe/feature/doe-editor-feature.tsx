import { useEffect, useMemo, useState } from 'react'
import { useUploadFile } from '#/hooks/use-customers'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import {
	type Doe,
	downloadDoePdf,
	previewDoePdf,
	useUpdateDoe,
} from '#/hooks/use-doe'
import { useFileUrls } from '#/hooks/use-file-url'
import { mutationErrorMessage } from '#/lib/api-error'
import { DoeSupplyPickerFeature } from '#/pages/doe/feature/doe-supply-picker-feature'
import { buildUpdatePayload, canSave, doeToForm } from '#/pages/doe/lib/payload'
import {
	enabledNumberedSections,
	isSectionEmpty,
	moveItem,
	newSection,
	suppliesOf,
	updateContent,
	updateSection,
	upsertSupply,
	withSupplies,
} from '#/pages/doe/lib/sections'
import {
	canSubmitSupply,
	EMPTY_SUPPLY_FORM,
	formToSupply,
	type SupplyFormValues,
	supplyToForm,
} from '#/pages/doe/lib/supply-form'
import type {
	DoeFile,
	DoeForm,
	DoeSupply,
	MakerLabel,
	PartyForm,
	PartyRole,
	SectionKind,
} from '#/pages/doe/types'
import { DoeEditorUI, INFO_TAB, PLAN_TAB } from '#/pages/doe/ui/doe-editor-ui'
import type { SectionActions } from '#/pages/doe/ui/doe-section-tab'
import { DoeSupplySheet } from '#/pages/doe/ui/doe-supply-sheet'

interface DoeEditorFeatureProps {
	doe: Doe
	organizationId: string
	organizationSlug: string
	projectId: string
	projectName: string
}

interface SheetState {
	sectionId: string
	supplyId: string
	sourceProductId: string | null
	isNew: boolean
	values: SupplyFormValues
}

function fileName(file: File): string {
	return file.name.replace(/\.[^.]+$/, '')
}

export function DoeEditorFeature({
	doe,
	organizationId,
	organizationSlug,
	projectId,
	projectName,
}: DoeEditorFeatureProps) {
	const initial = useMemo(() => doeToForm(doe), [doe])
	const [form, setForm] = useState<DoeForm>(initial)
	const dirty = useDirtyBaseline(initial, form)
	const [tab, setTab] = useState(INFO_TAB)
	const [sheet, setSheet] = useState<SheetState | null>(null)
	const [pickerSectionId, setPickerSectionId] = useState<string | null>(null)
	const [notice, setNotice] = useState<string | null>(null)
	const [pdfError, setPdfError] = useState<string | null>(null)
	const [isDownloading, setIsDownloading] = useState(false)
	const [uploadError, setUploadError] = useState<string | null>(null)
	const [uploadingSectionId, setUploadingSectionId] = useState<string | null>(
		null,
	)
	const [uploadingLogo, setUploadingLogo] = useState<PartyRole | null>(null)
	const [uploadingCover, setUploadingCover] = useState(false)
	const [sheetUploading, setSheetUploading] = useState<
		'photos' | 'datasheets' | null
	>(null)

	const updateDoe = useUpdateDoe()
	const uploadFile = useUploadFile()

	useEffect(() => {
		if (!notice) return
		const timer = setTimeout(() => setNotice(null), 4000)
		return () => clearTimeout(timer)
	}, [notice])

	const enabled = enabledNumberedSections(form.sections)
	const activeTab =
		tab === INFO_TAB ||
		tab === PLAN_TAB ||
		enabled.some(({ section }) => section.id === tab)
			? tab
			: PLAN_TAB

	const urlKeys = useMemo(() => {
		const keys = new Set<string>()
		if (form.coverPhotoKey) keys.add(form.coverPhotoKey)
		for (const party of form.parties) {
			if (party.logoKey) keys.add(party.logoKey)
		}
		for (const section of form.sections) {
			if (section.content.kind === 'photos') {
				for (const photo of section.content.photos) keys.add(photo.key)
			}
		}
		if (sheet) {
			for (const photo of sheet.values.photos) keys.add(photo.key)
			for (const key of sheet.values.datasheetKeys) keys.add(key)
		}
		return [...keys]
	}, [form.coverPhotoKey, form.parties, form.sections, sheet])
	const previews = useFileUrls(urlKeys)
	const fileUrls = useMemo(() => {
		const urls: Record<string, string | undefined> = {}
		for (const preview of previews) urls[preview.key] = preview.url
		return urls
	}, [previews])

	const setSections = (
		update: (sections: DoeForm['sections']) => DoeForm['sections'],
	) =>
		setForm((current) => ({ ...current, sections: update(current.sections) }))

	const uploadAll = async (files: File[]) => {
		setUploadError(null)
		const uploaded: Array<{ key: string; name: string }> = []
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
		await updateDoe.mutateAsync({
			path: { organization_id: organizationId, doe_id: doe.id },
			body: buildUpdatePayload(form, doe.updated_at),
		})
		dirty.commit(form)
		setNotice('Enregistré')
	}

	const runPdf = async (action: () => Promise<void>) => {
		setPdfError(null)
		setIsDownloading(true)
		try {
			await action()
		} catch (error) {
			setPdfError(mutationErrorMessage(error))
		} finally {
			setIsDownloading(false)
		}
	}

	const sectionActions: SectionActions = {
		uploadingSectionId,
		onMakerLabelChange: (sectionId: string, label: MakerLabel) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'supplies'
						? { ...content, maker_label: label }
						: content,
				),
			),
		onPickFromCatalog: (sectionId) => setPickerSectionId(sectionId),
		onCreateSupply: (sectionId) =>
			setSheet({
				sectionId,
				supplyId: crypto.randomUUID(),
				sourceProductId: null,
				isNew: true,
				values: EMPTY_SUPPLY_FORM,
			}),
		onEditSupply: (sectionId, supply) =>
			setSheet({
				sectionId,
				supplyId: supply.id,
				sourceProductId: supply.source_product_id ?? null,
				isNew: false,
				values: supplyToForm(supply),
			}),
		onMoveSupply: (sectionId, index, delta) =>
			setSections((sections) =>
				withSupplies(
					sections,
					sectionId,
					moveItem(suppliesOf(sections, sectionId), index, delta),
				),
			),
		onRemoveSupply: (sectionId, supply) =>
			setSections((sections) =>
				withSupplies(
					sections,
					sectionId,
					suppliesOf(sections, sectionId).filter(
						(entry) => entry.id !== supply.id,
					),
				),
			),
		onPerPageChange: (sectionId, perPage) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'photos'
						? { ...content, per_page: perPage }
						: content,
				),
			),
		onUploadPhotos: async (sectionId, files) => {
			setUploadingSectionId(sectionId)
			const uploaded = await uploadAll(files)
			setUploadingSectionId(null)
			const added: DoeFile[] = uploaded.map(({ key }) => ({
				key,
				caption: null,
			}))
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'photos'
						? { ...content, photos: [...content.photos, ...added] }
						: content,
				),
			)
		},
		onPhotoCaptionChange: (sectionId, index, caption) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'photos'
						? {
								...content,
								photos: content.photos.map((photo, position) =>
									position === index ? { ...photo, caption } : photo,
								),
							}
						: content,
				),
			),
		onRemovePhoto: (sectionId, index) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'photos'
						? {
								...content,
								photos: content.photos.filter(
									(_, position) => position !== index,
								),
							}
						: content,
				),
			),
		onUploadPlans: async (sectionId, files) => {
			setUploadingSectionId(sectionId)
			const uploaded = await uploadAll(files)
			setUploadingSectionId(null)
			const added: DoeFile[] = uploaded.map(({ key, name }) => ({
				key,
				caption: name,
			}))
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'plans'
						? { ...content, plans: [...content.plans, ...added] }
						: content,
				),
			)
		},
		onPlanCaptionChange: (sectionId, index, caption) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'plans'
						? {
								...content,
								plans: content.plans.map((plan, position) =>
									position === index ? { ...plan, caption } : plan,
								),
							}
						: content,
				),
			),
		onMovePlan: (sectionId, index, delta) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'plans'
						? { ...content, plans: moveItem(content.plans, index, delta) }
						: content,
				),
			),
		onRemovePlan: (sectionId, index) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'plans'
						? {
								...content,
								plans: content.plans.filter(
									(_, position) => position !== index,
								),
							}
						: content,
				),
			),
		onBlankCountChange: (sectionId, count) =>
			setSections((sections) =>
				updateContent(sections, sectionId, (content) =>
					content.kind === 'blank_pages'
						? { ...content, count: Math.min(20, Math.max(1, count)) }
						: content,
				),
			),
	}

	const submitSheet = () => {
		if (!sheet || !canSubmitSupply(sheet.values)) return
		const supply = formToSupply(
			{ id: sheet.supplyId, source_product_id: sheet.sourceProductId },
			sheet.values,
		)
		setSections((sections) =>
			withSupplies(
				sections,
				sheet.sectionId,
				upsertSupply(suppliesOf(sections, sheet.sectionId), supply),
			),
		)
		setSheet(null)
	}

	const sheetSection = sheet
		? form.sections.find((section) => section.id === sheet.sectionId)
		: undefined
	const sheetMakerLabel: MakerLabel =
		sheetSection?.content.kind === 'supplies'
			? sheetSection.content.maker_label
			: 'manufacturer'

	const pickerSupplies: DoeSupply[] = pickerSectionId
		? suppliesOf(form.sections, pickerSectionId)
		: []

	return (
		<>
			<DoeEditorUI
				header={{
					organizationSlug,
					projectId,
					projectName,
					title: form.title,
					revision: form.revision,
					lot: form.lot,
					isDirty: dirty.isDirty,
					isSaving: updateDoe.isPending,
					isDownloading,
					canSave: canSave(form),
					notice,
					onSave: () => {
						void save().catch(() => {})
					},
					onPreview: () =>
						void runPdf(() => previewDoePdf(organizationId, doe.id)),
					onDownload: () =>
						void runPdf(() =>
							downloadDoePdf(organizationId, doe.id, `DOE-${form.title}.pdf`),
						),
				}}
				form={form}
				activeTab={activeTab}
				error={mutationErrorMessage(updateDoe.error) ?? pdfError ?? uploadError}
				fileUrls={fileUrls}
				info={{
					isUploadingLogo: uploadingLogo,
					isUploadingCover: uploadingCover,
					onCoverPhotoUpload: async (file) => {
						setUploadingCover(true)
						const [uploaded] = await uploadAll([file])
						setUploadingCover(false)
						if (!uploaded) return
						setForm((current) => ({ ...current, coverPhotoKey: uploaded.key }))
					},
					onFieldChange: (patch) =>
						setForm((current) => ({ ...current, ...patch })),
					onPartyChange: (role: PartyRole, patch: Partial<PartyForm>) =>
						setForm((current) => ({
							...current,
							parties: current.parties.map((party) =>
								party.role === role ? { ...party, ...patch } : party,
							),
						})),
					onLogoUpload: async (role, file) => {
						setUploadingLogo(role)
						const [uploaded] = await uploadAll([file])
						setUploadingLogo(null)
						if (!uploaded) return
						setForm((current) => ({
							...current,
							parties: current.parties.map((party) =>
								party.role === role
									? { ...party, logoKey: uploaded.key }
									: party,
							),
						}))
					},
				}}
				plan={{
					removableSectionIds: new Set(
						form.sections.filter(isSectionEmpty).map((section) => section.id),
					),
					onToggle: (id, isEnabled) =>
						setSections((sections) =>
							updateSection(sections, id, { enabled: isEnabled }),
						),
					onRename: (id, title) =>
						setSections((sections) => updateSection(sections, id, { title })),
					onMove: (index, delta) =>
						setSections((sections) => moveItem(sections, index, delta)),
					onRemove: (id) => {
						setSections((sections) =>
							sections.filter((section) => section.id !== id),
						)
					},
					onAdd: (title: string, kind: SectionKind) => {
						const id = crypto.randomUUID()
						setSections((sections) => [
							...sections,
							newSection(id, title, kind),
						])
					},
				}}
				sectionActions={sectionActions}
				onTabChange={setTab}
			/>

			<DoeSupplySheet
				open={sheet !== null}
				isNew={sheet?.isNew ?? true}
				values={sheet?.values ?? EMPTY_SUPPLY_FORM}
				makerLabel={sheetMakerLabel}
				fileUrls={fileUrls}
				isUploadingPhotos={sheetUploading === 'photos'}
				isUploadingDatasheets={sheetUploading === 'datasheets'}
				error={uploadError}
				onOpenChange={(open) => {
					if (!open) setSheet(null)
				}}
				onChange={(patch) =>
					setSheet((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onUploadPhotos={async (files) => {
					setSheetUploading('photos')
					const uploaded = await uploadAll(files)
					setSheetUploading(null)
					setSheet((current) =>
						current
							? {
									...current,
									values: {
										...current.values,
										photos: [
											...current.values.photos,
											...uploaded.map(({ key }) => ({ key, caption: null })),
										],
									},
								}
							: current,
					)
				}}
				onUploadDatasheets={async (files) => {
					setSheetUploading('datasheets')
					const uploaded = await uploadAll(files)
					setSheetUploading(null)
					setSheet((current) =>
						current
							? {
									...current,
									values: {
										...current.values,
										datasheetKeys: [
											...current.values.datasheetKeys,
											...uploaded.map(({ key }) => key),
										],
									},
								}
							: current,
					)
				}}
				onSubmit={submitSheet}
			/>

			<DoeSupplyPickerFeature
				open={pickerSectionId !== null}
				organizationId={organizationId}
				doeId={doe.id}
				supplies={pickerSupplies}
				onSuppliesChange={(supplies) => {
					if (pickerSectionId) {
						setSections((sections) =>
							withSupplies(sections, pickerSectionId, supplies),
						)
					}
				}}
				onClose={() => setPickerSectionId(null)}
			/>
		</>
	)
}

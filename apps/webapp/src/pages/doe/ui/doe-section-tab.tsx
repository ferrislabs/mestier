import type {
	DoeSection,
	DoeSupply,
	MakerLabel,
	PhotosPerPage,
} from '#/pages/doe/types'
import { DoeBlankPagesTab } from '#/pages/doe/ui/doe-blank-pages-tab'
import { DoePhotosTab } from '#/pages/doe/ui/doe-photos-tab'
import { DoePlansTab } from '#/pages/doe/ui/doe-plans-tab'
import { DoeSuppliesTab } from '#/pages/doe/ui/doe-supplies-tab'

export interface SectionActions {
	uploadingSectionId: string | null
	onMakerLabelChange: (sectionId: string, label: MakerLabel) => void
	onPickFromCatalog: (sectionId: string) => void
	onCreateSupply: (sectionId: string) => void
	onEditSupply: (sectionId: string, supply: DoeSupply) => void
	onMoveSupply: (sectionId: string, index: number, delta: -1 | 1) => void
	onRemoveSupply: (sectionId: string, supply: DoeSupply) => void
	onPerPageChange: (sectionId: string, perPage: PhotosPerPage) => void
	onUploadPhotos: (sectionId: string, files: File[]) => void
	onPhotoCaptionChange: (
		sectionId: string,
		index: number,
		caption: string,
	) => void
	onRemovePhoto: (sectionId: string, index: number) => void
	onUploadPlans: (sectionId: string, files: File[]) => void
	onPlanCaptionChange: (
		sectionId: string,
		index: number,
		caption: string,
	) => void
	onMovePlan: (sectionId: string, index: number, delta: -1 | 1) => void
	onRemovePlan: (sectionId: string, index: number) => void
	onBlankCountChange: (sectionId: string, count: number) => void
}

interface DoeSectionTabProps {
	section: DoeSection
	number: number
	fileUrls: Record<string, string | undefined>
	actions: SectionActions
}

export function sectionItemCount(section: DoeSection): number {
	const { content } = section
	switch (content.kind) {
		case 'supplies':
			return content.supplies.length
		case 'photos':
			return content.photos.length
		case 'plans':
			return content.plans.length
		case 'blank_pages':
			return content.count
	}
}

export function DoeSectionTab({
	section,
	number,
	fileUrls,
	actions,
}: DoeSectionTabProps) {
	const { content } = section
	const isUploading = actions.uploadingSectionId === section.id

	switch (content.kind) {
		case 'supplies':
			return (
				<DoeSuppliesTab
					sectionNumber={number}
					title={section.title}
					makerLabel={content.maker_label}
					supplies={content.supplies}
					onMakerLabelChange={(label) =>
						actions.onMakerLabelChange(section.id, label)
					}
					onPickFromCatalog={() => actions.onPickFromCatalog(section.id)}
					onCreate={() => actions.onCreateSupply(section.id)}
					onEdit={(supply) => actions.onEditSupply(section.id, supply)}
					onMove={(index, delta) =>
						actions.onMoveSupply(section.id, index, delta)
					}
					onRemove={(supply) => actions.onRemoveSupply(section.id, supply)}
				/>
			)
		case 'photos':
			return (
				<DoePhotosTab
					sectionNumber={number}
					title={section.title}
					perPage={content.per_page}
					photos={content.photos}
					fileUrls={fileUrls}
					isUploading={isUploading}
					onPerPageChange={(perPage) =>
						actions.onPerPageChange(section.id, perPage)
					}
					onUpload={(files) => actions.onUploadPhotos(section.id, files)}
					onCaptionChange={(index, caption) =>
						actions.onPhotoCaptionChange(section.id, index, caption)
					}
					onRemove={(index) => actions.onRemovePhoto(section.id, index)}
				/>
			)
		case 'plans':
			return (
				<DoePlansTab
					sectionNumber={number}
					title={section.title}
					plans={content.plans}
					isUploading={isUploading}
					onUpload={(files) => actions.onUploadPlans(section.id, files)}
					onCaptionChange={(index, caption) =>
						actions.onPlanCaptionChange(section.id, index, caption)
					}
					onMove={(index, delta) =>
						actions.onMovePlan(section.id, index, delta)
					}
					onRemove={(index) => actions.onRemovePlan(section.id, index)}
				/>
			)
		case 'blank_pages':
			return (
				<DoeBlankPagesTab
					sectionNumber={number}
					title={section.title}
					count={content.count}
					onCountChange={(count) =>
						actions.onBlankCountChange(section.id, count)
					}
				/>
			)
	}
}

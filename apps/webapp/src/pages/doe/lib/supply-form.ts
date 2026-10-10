import type { DoeFile, DoeSupply } from '#/pages/doe/types'

export interface SupplyFormValues {
	title: string
	dpgfRef: string
	destination: string
	designation: string
	supplierName: string
	supplierAddress: string
	supplierPhone: string
	supplierEmail: string
	maker: string
	specs: string
	photos: DoeFile[]
	datasheetKeys: string[]
}

export const EMPTY_SUPPLY_FORM: SupplyFormValues = {
	title: '',
	dpgfRef: '',
	destination: '',
	designation: '',
	supplierName: '',
	supplierAddress: '',
	supplierPhone: '',
	supplierEmail: '',
	maker: '',
	specs: '',
	photos: [],
	datasheetKeys: [],
}

export function supplyToForm(supply: DoeSupply): SupplyFormValues {
	return {
		title: supply.title ?? '',
		dpgfRef: supply.dpgf_ref ?? '',
		destination: supply.destination ?? '',
		designation: supply.designation,
		supplierName: supply.supplier?.name ?? '',
		supplierAddress: supply.supplier?.address ?? '',
		supplierPhone: supply.supplier?.phone ?? '',
		supplierEmail: supply.supplier?.email ?? '',
		maker: supply.maker ?? '',
		specs: supply.specs ?? '',
		photos: supply.photos,
		datasheetKeys: supply.datasheet_keys,
	}
}

function orNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function formToSupply(
	base: Pick<DoeSupply, 'id' | 'source_product_id'>,
	form: SupplyFormValues,
): DoeSupply {
	const supplierName = form.supplierName.trim()
	return {
		id: base.id,
		source_product_id: base.source_product_id ?? null,
		title: orNull(form.title),
		designation: form.designation.trim(),
		dpgf_ref: orNull(form.dpgfRef),
		destination: orNull(form.destination),
		supplier:
			supplierName === ''
				? null
				: {
						name: supplierName,
						address: orNull(form.supplierAddress),
						phone: orNull(form.supplierPhone),
						email: orNull(form.supplierEmail),
					},
		maker: orNull(form.maker),
		specs: orNull(form.specs),
		photos: form.photos.map((photo) => ({
			key: photo.key,
			caption: orNull(photo.caption ?? ''),
		})),
		datasheet_keys: form.datasheetKeys,
	}
}

export function canSubmitSupply(form: SupplyFormValues): boolean {
	return form.designation.trim() !== ''
}

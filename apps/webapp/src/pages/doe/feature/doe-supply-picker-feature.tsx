import { useState } from 'react'
import { useDraftDoeSupply } from '#/hooks/use-doe'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { mutationErrorMessage } from '#/lib/api-error'
import { pickedProductIds, toggleProduct } from '#/pages/doe/lib/picker'
import type { DoeSupply } from '#/pages/doe/types'
import { DoeSupplyPickerDialog } from '#/pages/doe/ui/doe-supply-picker-dialog'

interface DoeSupplyPickerFeatureProps {
	open: boolean
	organizationId: string
	doeId: string
	supplies: DoeSupply[]
	onSuppliesChange: (supplies: DoeSupply[]) => void
	onClose: () => void
}

export function DoeSupplyPickerFeature({
	open,
	organizationId,
	doeId,
	supplies,
	onSuppliesChange,
	onClose,
}: DoeSupplyPickerFeatureProps) {
	const [search, setSearch] = useState('')
	const [isDrafting, setIsDrafting] = useState(false)
	const draft = useDraftDoeSupply()
	const { products } = useReferenceCatalog(organizationId, {
		members: false,
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		customUnits: false,
		products: open,
	})

	const needle = search.trim().toLowerCase()
	const rows = (products.data?.data ?? []).filter((product) =>
		needle === '' ? true : product.name.toLowerCase().includes(needle),
	)

	const onToggle = async (productId: string, checked: boolean) => {
		setIsDrafting(true)
		try {
			const next = await toggleProduct(
				supplies,
				productId,
				checked,
				async (id) => {
					const response = await draft.mutateAsync({
						path: { organization_id: organizationId, doe_id: doeId },
						body: { product_id: id },
					})
					return response.data
				},
			)
			onSuppliesChange(next)
		} catch {
		} finally {
			setIsDrafting(false)
		}
	}

	return (
		<DoeSupplyPickerDialog
			open={open}
			products={rows}
			isLoading={products.isLoading}
			search={search}
			pickedIds={pickedProductIds(supplies)}
			isDrafting={isDrafting}
			error={
				mutationErrorMessage(draft.error) ??
				mutationErrorMessage(products.error)
			}
			onSearchChange={setSearch}
			onToggle={(productId, checked) => {
				void onToggle(productId, checked)
			}}
			onOpenChange={(next) => {
				if (!next) onClose()
			}}
		/>
	)
}

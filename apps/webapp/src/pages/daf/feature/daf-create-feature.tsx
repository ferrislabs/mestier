import { useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useCreateDaf } from '#/hooks/use-daf'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { DafCreateDialog } from '#/pages/daf/ui/daf-create-dialog'
import { DoeSupplyPickerDialog } from '#/pages/doe/ui/doe-supply-picker-dialog'

export function DafCreateFeature({
	open,
	organizationId,
	organizationSlug,
	projectId,
	onClose,
}: {
	open: boolean
	organizationId: string
	organizationSlug: string
	projectId: string
	onClose: () => void
}) {
	const [designation, setDesignation] = useState('')
	const [pickingProduct, setPickingProduct] = useState(false)
	const [search, setSearch] = useState('')
	const create = useCreateDaf()
	const navigate = useNavigate()
	const { reset: resetCreate } = create
	const { products } = useReferenceCatalog(organizationId, {
		members: false,
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		customUnits: false,
		products: open && pickingProduct,
	})

	useEffect(() => {
		if (open) return
		setDesignation('')
		setPickingProduct(false)
		setSearch('')
		resetCreate()
	}, [open, resetCreate])

	const submit = async (body: {
		designation?: string
		product_id?: string
	}) => {
		try {
			const response = await create.mutateAsync({
				path: { organization_id: organizationId, project_id: projectId },
				body,
			})
			onClose()
			void navigate({
				to: buildOrgPath(organizationSlug, '/projet/$projectId/daf/$dafId'),
				params: { projectId, dafId: response.data.id },
			})
		} catch {}
	}

	const needle = search.trim().toLowerCase()
	const rows = (products.data?.data ?? []).filter((product) =>
		needle === '' ? true : product.name.toLowerCase().includes(needle),
	)
	const error = mutationErrorMessage(create.error)

	return (
		<>
			<DafCreateDialog
				open={open && !pickingProduct}
				designation={designation}
				isPending={create.isPending}
				error={error}
				onDesignationChange={setDesignation}
				onPickProduct={() => setPickingProduct(true)}
				onSubmit={() => {
					void submit({ designation: designation.trim() })
				}}
				onOpenChange={(next) => {
					if (!next) onClose()
				}}
			/>
			<DoeSupplyPickerDialog
				open={open && pickingProduct}
				products={rows}
				isLoading={products.isLoading}
				search={search}
				pickedIds={new Set()}
				isDrafting={create.isPending}
				description="Choisir un produit crée la DAF avec sa fiche : fournisseur, caractéristiques, photo et pièces jointes."
				error={error ?? mutationErrorMessage(products.error)}
				onSearchChange={setSearch}
				onToggle={(productId) => {
					void submit({ product_id: productId })
				}}
				onOpenChange={(next) => {
					if (!next) setPickingProduct(false)
				}}
			/>
		</>
	)
}

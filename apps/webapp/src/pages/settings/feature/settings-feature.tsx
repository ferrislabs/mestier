import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	useCreateEquipment,
	useDeleteEquipment,
	useReferenceCatalog,
	useUpdateEquipment,
} from '#/hooks/use-reference-catalog'
import type { EquipmentFormValues } from '#/pages/settings/types'
import { SettingsUI } from '#/pages/settings/ui/settings-ui'

export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le référentiel nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<SettingsCatalog
			key={activeOrganization.id}
			organization={activeOrganization}
		/>
	)
}

interface SettingsCatalogProps {
	organization: Organization
}

function SettingsCatalog({ organization }: SettingsCatalogProps) {
	const organizationId = organization.id
	const catalog = useReferenceCatalog(organizationId, {
		employees: false,
		serviceRates: false,
		products: false,
	})
	const createEquipment = useCreateEquipment(organizationId)
	const updateEquipment = useUpdateEquipment()
	const deleteEquipment = useDeleteEquipment()

	const equipmentForm = useForm({
		defaultValues: { name: '', hourlyRate: '' } satisfies EquipmentFormValues,
		onSubmit: async ({ value }) => {
			await createEquipment.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					hourly_rate_cents: eurosToCents(value.hourlyRate),
				},
			})
			equipmentForm.reset()
		},
	})

	const isLoading = catalog.equipment.isLoading

	const error =
		catalog.equipment.error ??
		createEquipment.error ??
		updateEquipment.error ??
		deleteEquipment.error

	return (
		<equipmentForm.Subscribe selector={(state) => state.values}>
			{(equipmentValues) => (
				<SettingsUI
					organization={organization}
					isLoading={isLoading}
					error={error?.message ?? null}
					data={{
						equipment: catalog.equipment.data?.data ?? [],
					}}
					equipmentForm={{
						values: equipmentValues,
						isPending: createEquipment.isPending,
						onChange: (patch) => {
							for (const key of Object.keys(
								patch,
							) as (keyof EquipmentFormValues)[]) {
								equipmentForm.setFieldValue(key, patch[key] ?? '')
							}
						},
						onSubmit: () => void equipmentForm.handleSubmit(),
					}}
					onUpdateEquipment={(equipment, values) =>
						updateEquipment.mutateAsync({
							path: { equipment_id: equipment.id },
							body: {
								name: values.name.trim(),
								hourly_rate_cents: eurosToCents(values.hourlyRate),
							},
						})
					}
					onDeleteEquipment={(equipment) =>
						deleteEquipment.mutateAsync({
							path: { equipment_id: equipment.id },
						})
					}
				/>
			)}
		</equipmentForm.Subscribe>
	)
}

function eurosToCents(value: string): number {
	const normalized = value.replace(',', '.').trim()
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) {
		return 0
	}
	return Math.round(parsed * 100)
}

import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Organization,
	useUpdateOrganization,
} from '#/hooks/use-organizations'
import {
	useCreateEmployee,
	useCreateEquipment,
	useDeleteEmployee,
	useDeleteEquipment,
	useReferenceCatalog,
	useUpdateEmployee,
	useUpdateEquipment,
} from '#/hooks/use-reference-catalog'
import type {
	EmployeeFormValues,
	EquipmentFormValues,
	OrganizationFormValues,
} from '#/pages/settings/types'
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
		serviceRates: false,
		products: false,
	})
	const updateOrganization = useUpdateOrganization(organizationId)
	const createEmployee = useCreateEmployee(organizationId)
	const updateEmployee = useUpdateEmployee()
	const deleteEmployee = useDeleteEmployee()
	const createEquipment = useCreateEquipment(organizationId)
	const updateEquipment = useUpdateEquipment()
	const deleteEquipment = useDeleteEquipment()

	const employeeForm = useForm({
		defaultValues: {
			name: '',
			hourlyRate: '',
			userId: '',
		} satisfies EmployeeFormValues,
		onSubmit: async ({ value }) => {
			await createEmployee.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					hourly_rate_cents: eurosToCents(value.hourlyRate),
					user_id: value.userId.trim() || null,
				},
			})
			employeeForm.reset()
		},
	})

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

	const organizationForm = useForm({
		defaultValues: {
			name: organization.name,
			slug: organization.slug,
		} satisfies OrganizationFormValues,
		onSubmit: async ({ value }) => {
			await updateOrganization.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					slug: normalizeSlug(value.slug),
				},
			})
		},
	})

	const isLoading = catalog.employees.isLoading || catalog.equipment.isLoading

	const error =
		catalog.employees.error ??
		catalog.equipment.error ??
		createEmployee.error ??
		updateEmployee.error ??
		deleteEmployee.error ??
		createEquipment.error ??
		updateEquipment.error ??
		deleteEquipment.error ??
		updateOrganization.error

	return (
		<organizationForm.Subscribe selector={(state) => state.values}>
			{(organizationValues) => (
				<employeeForm.Subscribe selector={(state) => state.values}>
					{(employeeValues) => (
						<equipmentForm.Subscribe selector={(state) => state.values}>
							{(equipmentValues) => (
								<SettingsUI
									organization={organization}
									isLoading={isLoading}
									error={error?.message ?? null}
									data={{
										employees: catalog.employees.data?.data ?? [],
										equipment: catalog.equipment.data?.data ?? [],
									}}
									organizationForm={{
										values: organizationValues,
										isPending: updateOrganization.isPending,
										onChange: (patch) => {
											for (const key of Object.keys(
												patch,
											) as (keyof OrganizationFormValues)[]) {
												organizationForm.setFieldValue(key, patch[key] ?? '')
											}
										},
										onSubmit: () => void organizationForm.handleSubmit(),
									}}
									employeeForm={{
										values: employeeValues,
										isPending: createEmployee.isPending,
										onChange: (patch) => {
											for (const key of Object.keys(
												patch,
											) as (keyof EmployeeFormValues)[]) {
												employeeForm.setFieldValue(key, patch[key] ?? '')
											}
										},
										onSubmit: () => void employeeForm.handleSubmit(),
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
									onUpdateEmployee={(employee, values) =>
										updateEmployee.mutateAsync({
											path: { employee_id: employee.id },
											body: {
												name: values.name.trim(),
												hourly_rate_cents: eurosToCents(values.hourlyRate),
												user_id: values.userId.trim() || null,
											},
										})
									}
									onDeleteEmployee={(employee) =>
										deleteEmployee.mutateAsync({
											path: { employee_id: employee.id },
										})
									}
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
					)}
				</employeeForm.Subscribe>
			)}
		</organizationForm.Subscribe>
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

function normalizeSlug(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
		.replace(/^-|-$/g, '')
}

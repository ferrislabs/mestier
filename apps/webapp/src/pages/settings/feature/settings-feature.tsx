import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { useMyOrganizations } from '#/hooks/use-organizations'
import {
	useCreateEmployee,
	useCreateEquipment,
	useCreateServiceRate,
	useDeleteEmployee,
	useDeleteEquipment,
	useDeleteServiceRate,
	useReferenceCatalog,
	useUpdateEmployee,
	useUpdateEquipment,
	useUpdateServiceRate,
} from '#/hooks/use-reference-catalog'
import type {
	EmployeeFormValues,
	EquipmentFormValues,
	ServiceRateFormValues,
} from '#/pages/settings/types'
import { SettingsUI } from '#/pages/settings/ui/settings-ui'

export function SettingsFeature() {
	const organizations = useMyOrganizations()
	const organization = organizations.data?.data?.[0]

	if (organizations.isLoading) {
		return <SettingsUI.Loading />
	}

	if (organizations.isError || !organization) {
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
				<Button onClick={() => void organizations.refetch()} variant="outline">
					Réessayer
				</Button>
			</div>
		)
	}

	return (
		<SettingsCatalog
			organizationId={organization.id}
			organizationName={organization.name}
		/>
	)
}

interface SettingsCatalogProps {
	organizationId: string
	organizationName: string
}

function SettingsCatalog({
	organizationId,
	organizationName,
}: SettingsCatalogProps) {
	const catalog = useReferenceCatalog(organizationId)
	const createEmployee = useCreateEmployee(organizationId)
	const updateEmployee = useUpdateEmployee()
	const deleteEmployee = useDeleteEmployee()
	const createEquipment = useCreateEquipment(organizationId)
	const updateEquipment = useUpdateEquipment()
	const deleteEquipment = useDeleteEquipment()
	const createServiceRate = useCreateServiceRate(organizationId)
	const updateServiceRate = useUpdateServiceRate()
	const deleteServiceRate = useDeleteServiceRate()

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

	const serviceRateForm = useForm({
		defaultValues: {
			label: '',
			unit: 'HOUR',
			rate: '',
		} satisfies ServiceRateFormValues,
		onSubmit: async ({ value }) => {
			await createServiceRate.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					label: value.label.trim(),
					unit: value.unit,
					rate_cents: eurosToCents(value.rate),
				},
			})
			serviceRateForm.reset()
		},
	})

	const isLoading =
		catalog.employees.isLoading ||
		catalog.equipment.isLoading ||
		catalog.serviceRates.isLoading

	const error =
		catalog.employees.error ??
		catalog.equipment.error ??
		catalog.serviceRates.error ??
		createEmployee.error ??
		updateEmployee.error ??
		deleteEmployee.error ??
		createEquipment.error ??
		updateEquipment.error ??
		deleteEquipment.error ??
		createServiceRate.error ??
		updateServiceRate.error ??
		deleteServiceRate.error

	return (
		<employeeForm.Subscribe selector={(state) => state.values}>
			{(employeeValues) => (
				<equipmentForm.Subscribe selector={(state) => state.values}>
					{(equipmentValues) => (
						<serviceRateForm.Subscribe selector={(state) => state.values}>
							{(serviceRateValues) => (
								<SettingsUI
									organizationName={organizationName}
									isLoading={isLoading}
									error={error?.message ?? null}
									data={{
										employees: catalog.employees.data?.data ?? [],
										equipment: catalog.equipment.data?.data ?? [],
										serviceRates: catalog.serviceRates.data?.data ?? [],
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
									serviceRateForm={{
										values: serviceRateValues,
										isPending: createServiceRate.isPending,
										onChange: (patch) => {
											for (const key of Object.keys(
												patch,
											) as (keyof ServiceRateFormValues)[]) {
												serviceRateForm.setFieldValue(key, patch[key] as never)
											}
										},
										onSubmit: () => void serviceRateForm.handleSubmit(),
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
									onUpdateServiceRate={(serviceRate, values) =>
										updateServiceRate.mutateAsync({
											path: { service_rate_id: serviceRate.id },
											body: {
												label: values.label.trim(),
												unit: values.unit,
												rate_cents: eurosToCents(values.rate),
											},
										})
									}
									onDeleteServiceRate={(serviceRate) =>
										deleteServiceRate.mutateAsync({
											path: { service_rate_id: serviceRate.id },
										})
									}
								/>
							)}
						</serviceRateForm.Subscribe>
					)}
				</equipmentForm.Subscribe>
			)}
		</employeeForm.Subscribe>
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

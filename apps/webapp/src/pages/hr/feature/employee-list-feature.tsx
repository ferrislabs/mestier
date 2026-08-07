import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateEmployee,
	useDeleteEmployee,
	useReferenceCatalog,
	useUpdateEmployee,
} from '#/hooks/use-reference-catalog'
import { type EmployeeFormValues, employeeDisplayName } from '#/pages/hr/types'
import {
	type EmployeeDraft,
	EmployeeListUI,
} from '#/pages/hr/ui/employee-list-ui'

export function EmployeeListFeature() {
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
						L’annuaire des employés nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<EmployeeDirectory
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
		/>
	)
}

interface EmployeeDirectoryProps {
	organizationId: string
	organizationName: string
}

function EmployeeDirectory({
	organizationId,
	organizationName,
}: EmployeeDirectoryProps) {
	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const createEmployee = useCreateEmployee(organizationId)
	const updateEmployee = useUpdateEmployee()
	const deleteEmployee = useDeleteEmployee()

	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<EmployeeDraft | null>(null)
	const [isSaving, setIsSaving] = useState(false)

	const employeeForm = useForm({
		defaultValues: {
			lastName: '',
			firstName: '',
			hourlyRate: '',
			userId: '',
		} satisfies EmployeeFormValues,
		onSubmit: async ({ value }) => {
			await createEmployee.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					last_name: value.lastName.trim(),
					first_name: value.firstName.trim() || null,
					hourly_rate_cents: eurosToCents(value.hourlyRate),
					user_id: value.userId.trim() || null,
				},
			})
			employeeForm.reset()
		},
	})

	const employees = catalog.employees.data?.data ?? []
	const normalizedSearch = search.trim().toLowerCase()
	const filteredEmployees = employees.filter((employee) =>
		employeeDisplayName(employee).toLowerCase().includes(normalizedSearch),
	)

	const isLoading = catalog.employees.isLoading

	const error =
		catalog.employees.error ??
		createEmployee.error ??
		updateEmployee.error ??
		deleteEmployee.error

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			const employee = employees.find((item) => item.id === draft.id)
			if (employee) {
				await updateEmployee.mutateAsync({
					path: { employee_id: employee.id },
					body: {
						last_name: draft.values.lastName.trim(),
						first_name: draft.values.firstName.trim() || null,
						hourly_rate_cents: eurosToCents(draft.values.hourlyRate),
						user_id: draft.values.userId.trim() || null,
					},
				})
			}
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<employeeForm.Subscribe selector={(state) => state.values}>
			{(employeeValues) => (
				<EmployeeListUI
					organizationName={organizationName}
					isLoading={isLoading}
					error={error?.message ?? null}
					data={{ employees: filteredEmployees }}
					search={search}
					onSearchChange={setSearch}
					createForm={{
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
					draft={draft}
					isSaving={isSaving}
					onEdit={(employee) =>
						setDraft({
							id: employee.id,
							values: {
								lastName: employee.last_name,
								firstName: employee.first_name ?? '',
								hourlyRate: centsToEuros(employee.hourly_rate_cents),
								userId: employee.user_id ?? '',
							},
						})
					}
					onDraftChange={(values) =>
						setDraft((current) => (current ? { ...current, values } : current))
					}
					onCancelEdit={() => setDraft(null)}
					onSaveEdit={handleSaveDraft}
					onDeleteEmployee={(employee) =>
						deleteEmployee.mutateAsync({ path: { employee_id: employee.id } })
					}
				/>
			)}
		</employeeForm.Subscribe>
	)
}

/**
 * An empty field means "rate not set", not "free". Collapsing it to 0 would
 * feed a wrong cost into the profitability computation instead of an absent
 * one, which is the whole reason the column is nullable.
 */
function eurosToCents(value: string): number | null {
	const normalized = value.replace(',', '.').trim()
	if (normalized === '') {
		return null
	}
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) {
		return null
	}
	return Math.round(parsed * 100)
}

function centsToEuros(value: number | null | undefined): string {
	if (value === null || value === undefined) {
		return ''
	}
	return (value / 100).toFixed(2).replace('.', ',')
}

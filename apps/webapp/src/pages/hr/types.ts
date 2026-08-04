import type { Employee } from '#/hooks/use-reference-catalog'

export interface EmployeeFormValues {
	name: string
	hourlyRate: string
	userId: string
}

export interface EmployeeListData {
	employees: Employee[]
}

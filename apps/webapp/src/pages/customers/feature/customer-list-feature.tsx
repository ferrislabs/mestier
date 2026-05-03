import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { MOCK_CUSTOMERS } from '#/pages/customers/mocks'
import type { Customer } from '#/pages/customers/types'
import { CustomerListUI } from '#/pages/customers/ui/customer-list-ui'

export function CustomerListFeature() {
	const navigate = useNavigate()
	const [customers] = useState<Customer[]>(MOCK_CUSTOMERS)
	const [isLoading] = useState(false)

	const handleAdd = () => {
		console.log('[customers] add')
	}

	const handleEdit = (customer: Customer) => {
		void navigate({
			to: '/customers/$customerId',
			params: { customerId: customer.id },
		})
	}

	const handleDelete = (customer: Customer) => {
		console.log('[customers] delete', customer.id)
	}

	return (
		<CustomerListUI
			customers={customers}
			isLoading={isLoading}
			onAdd={handleAdd}
			onEdit={handleEdit}
			onDelete={handleDelete}
		/>
	)
}

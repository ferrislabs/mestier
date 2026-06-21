export namespace Schemas {
	// <Schemas>
	export type CreateCustomerContactRequest = {
		email?: (string | null) | undefined
		first_name: string
		is_primary: boolean
		last_name: string
		phone?: (string | null) | undefined
		role?: (string | null) | undefined
	}
	export type CreateCustomerContextRequest = {
		address_line?: (string | null) | undefined
		city?: (string | null) | undefined
		label: string
		photo_key?: (string | null) | undefined
		postal_code?: (string | null) | undefined
	}
	export type CustomerStatus = 'PROSPECT' | 'CLIENT' | 'ARCHIVED'
	export type CreateCustomerRequest = {
		email?: (string | null) | undefined
		first_name: string
		last_name: string
		phone?: (string | null) | undefined
		status: CustomerStatus
	}
	export type UserId = string
	export type CreateEmployeeRequest = {
		hourly_rate_cents: number
		name: string
		user_id?: (null | UserId) | undefined
	}
	export type CreateEquipmentRequest = {
		hourly_rate_cents: number
		name: string
	}
	export type CreateOrganizationRequest = { name: string; slug: string }
	export type ServiceRateUnit = 'HOUR' | 'ML' | 'M2'
	export type CreateProductRequest = {
		description?: (string | null) | undefined
		name: string
		sku?: (string | null) | undefined
		unit: ServiceRateUnit
		unit_price_cents: number
	}
	export type CustomerContextId = string
	export type CustomerId = string
	export type ServiceRateId = string
	export type QuoteLineRequest = {
		label: string
		notes?: (string | null) | undefined
		photo_keys: Array<string>
		quantity: string
		service_rate_id?: (null | ServiceRateId) | undefined
		unit: ServiceRateUnit
		unit_price_cents: number
	}
	export type CreateQuoteRequest = {
		customer_context_id: CustomerContextId
		customer_id: CustomerId
		lines: Array<QuoteLineRequest>
		title: string
	}
	export type CreateServiceRateRequest = {
		label: string
		rate_cents: number
		unit: ServiceRateUnit
	}
	export type CustomerContactId = string
	export type CustomerContactResponse = {
		created_at: string
		customer_id: CustomerId
		email?: (string | null) | undefined
		first_name: string
		id: CustomerContactId
		is_primary: boolean
		last_name: string
		phone?: (string | null) | undefined
		role?: (string | null) | undefined
		updated_at: string
	}
	export type CustomerContextResponse = {
		address_line?: (string | null) | undefined
		city?: (string | null) | undefined
		created_at: string
		customer_id: CustomerId
		id: CustomerContextId
		label: string
		photo_key?: (string | null) | undefined
		postal_code?: (string | null) | undefined
		updated_at: string
	}
	export type OrganizationId = string
	export type CustomerResponse = {
		created_at: string
		email?: (string | null) | undefined
		first_name: string
		id: CustomerId
		last_name: string
		organization_id: OrganizationId
		phone?: (string | null) | undefined
		status: CustomerStatus
		updated_at: string
	}
	export type EmployeeId = string
	export type EmployeeResponse = {
		created_at: string
		hourly_rate_cents: number
		id: EmployeeId
		name: string
		organization_id: OrganizationId
		updated_at: string
		user_id?: (null | UserId) | undefined
	}
	export type EquipmentId = string
	export type EquipmentResponse = {
		created_at: string
		hourly_rate_cents: number
		id: EquipmentId
		name: string
		organization_id: OrganizationId
		updated_at: string
	}
	export type FileUploadResponse = {
		key: string
		mime_type: string
		size_bytes: number
	}
	export type OrganizationResponse = {
		created_at: string
		id: OrganizationId
		name: string
		owner_id: UserId
		slug: string
		updated_at: string
	}
	export type PaginationMetadata = {
		current_page: number
		first_page: number
		is_empty: boolean
		last_page?: (number | null) | undefined
		next_page?: (number | null) | undefined
		per_page: number
		prev_page?: (number | null) | undefined
		total?: (number | null) | undefined
	}
	export type ProductId = string
	export type ProductResponse = {
		created_at: string
		description?: (string | null) | undefined
		id: ProductId
		name: string
		organization_id: OrganizationId
		sku?: (string | null) | undefined
		unit: ServiceRateUnit
		unit_price_cents: number
		updated_at: string
	}
	export type QuoteId = string
	export type QuoteLineId = string
	export type QuoteLineResponse = {
		created_at: string
		id: QuoteLineId
		label: string
		notes?: (string | null) | undefined
		organization_id: OrganizationId
		photo_keys: Array<string>
		quantity: string
		quote_id: QuoteId
		service_rate_id?: (null | ServiceRateId) | undefined
		unit: ServiceRateUnit
		unit_price_cents: number
		updated_at: string
	}
	export type QuoteStatus =
		| 'DRAFT'
		| 'SENT'
		| 'ACCEPTED'
		| 'DECLINED'
		| 'CANCELLED'
	export type QuoteResponse = {
		created_at: string
		customer_context_id: CustomerContextId
		customer_id: CustomerId
		id: QuoteId
		lines: Array<QuoteLineResponse>
		organization_id: OrganizationId
		reference: string
		status: QuoteStatus
		title: string
		total_cents: number
		updated_at: string
	}
	export type ServiceRateResponse = {
		created_at: string
		id: ServiceRateId
		label: string
		organization_id: OrganizationId
		rate_cents: number
		unit: ServiceRateUnit
		updated_at: string
	}
	export type UpdateCustomerContactRequest = {
		email?: (string | null) | undefined
		first_name: string
		is_primary: boolean
		last_name: string
		phone?: (string | null) | undefined
		role?: (string | null) | undefined
	}
	export type UpdateCustomerContextRequest = {
		address_line?: (string | null) | undefined
		city?: (string | null) | undefined
		label: string
		photo_key?: (string | null) | undefined
		postal_code?: (string | null) | undefined
	}
	export type UpdateCustomerRequest = {
		email?: (string | null) | undefined
		first_name: string
		last_name: string
		phone?: (string | null) | undefined
		status: CustomerStatus
	}
	export type UpdateEmployeeRequest = {
		hourly_rate_cents: number
		name: string
		user_id?: (null | UserId) | undefined
	}
	export type UpdateEquipmentRequest = {
		hourly_rate_cents: number
		name: string
	}
	export type UpdateOrganizationRequest = { name: string; slug: string }
	export type UpdateProductRequest = {
		description?: (string | null) | undefined
		name: string
		sku?: (string | null) | undefined
		unit: ServiceRateUnit
		unit_price_cents: number
	}
	export type UpdateQuoteRequest = {
		customer_context_id: CustomerContextId
		customer_id: CustomerId
		lines: Array<QuoteLineRequest>
		status: QuoteStatus
		title: string
	}
	export type UpdateQuoteStatusRequest = { status: QuoteStatus }
	export type UpdateServiceRateRequest = {
		label: string
		rate_cents: number
		unit: ServiceRateUnit
	}

	// </Schemas>
}

export namespace Endpoints {
	// <Endpoints>

	export type get_GetCustomerContact = {
		method: 'GET'
		path: '/api/v1/customer-contacts/{customer_contact_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_contact_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					customer_id: Schemas.CustomerId
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerContactId
					is_primary: boolean
					last_name: string
					phone?: (string | null) | undefined
					role?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteCustomerContact = {
		method: 'DELETE'
		path: '/api/v1/customer-contacts/{customer_contact_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_contact_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateCustomerContact = {
		method: 'PATCH'
		path: '/api/v1/customer-contacts/{customer_contact_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_contact_id: string }

			body: Schemas.UpdateCustomerContactRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					customer_id: Schemas.CustomerId
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerContactId
					is_primary: boolean
					last_name: string
					phone?: (string | null) | undefined
					role?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_GetCustomerContext = {
		method: 'GET'
		path: '/api/v1/customer-contexts/{customer_context_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_context_id: string }
		}
		responses: {
			200: {
				data: {
					address_line?: (string | null) | undefined
					city?: (string | null) | undefined
					created_at: string
					customer_id: Schemas.CustomerId
					id: Schemas.CustomerContextId
					label: string
					photo_key?: (string | null) | undefined
					postal_code?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteCustomerContext = {
		method: 'DELETE'
		path: '/api/v1/customer-contexts/{customer_context_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_context_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateCustomerContext = {
		method: 'PATCH'
		path: '/api/v1/customer-contexts/{customer_context_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_context_id: string }

			body: Schemas.UpdateCustomerContextRequest
		}
		responses: {
			200: {
				data: {
					address_line?: (string | null) | undefined
					city?: (string | null) | undefined
					created_at: string
					customer_id: Schemas.CustomerId
					id: Schemas.CustomerContextId
					label: string
					photo_key?: (string | null) | undefined
					postal_code?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_GetCustomer = {
		method: 'GET'
		path: '/api/v1/customers/{customer_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerId
					last_name: string
					organization_id: Schemas.OrganizationId
					phone?: (string | null) | undefined
					status: Schemas.CustomerStatus
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteCustomer = {
		method: 'DELETE'
		path: '/api/v1/customers/{customer_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateCustomer = {
		method: 'PATCH'
		path: '/api/v1/customers/{customer_id}'
		requestFormat: 'json'
		parameters: {
			path: { customer_id: string }

			body: Schemas.UpdateCustomerRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerId
					last_name: string
					organization_id: Schemas.OrganizationId
					phone?: (string | null) | undefined
					status: Schemas.CustomerStatus
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_ListCustomerContacts = {
		method: 'GET'
		path: '/api/v1/customers/{customer_id}/contacts'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { customer_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					customer_id: Schemas.CustomerId
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerContactId
					is_primary: boolean
					last_name: string
					phone?: (string | null) | undefined
					role?: (string | null) | undefined
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type post_CreateCustomerContact = {
		method: 'POST'
		path: '/api/v1/customers/{customer_id}/contacts'
		requestFormat: 'json'
		parameters: {
			path: { customer_id: string }

			body: Schemas.CreateCustomerContactRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					customer_id: Schemas.CustomerId
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerContactId
					is_primary: boolean
					last_name: string
					phone?: (string | null) | undefined
					role?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_ListCustomerContexts = {
		method: 'GET'
		path: '/api/v1/customers/{customer_id}/customer-contexts'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { customer_id: string }
		}
		responses: {
			200: {
				data: Array<{
					address_line?: (string | null) | undefined
					city?: (string | null) | undefined
					created_at: string
					customer_id: Schemas.CustomerId
					id: Schemas.CustomerContextId
					label: string
					photo_key?: (string | null) | undefined
					postal_code?: (string | null) | undefined
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type post_CreateCustomerContext = {
		method: 'POST'
		path: '/api/v1/customers/{customer_id}/customer-contexts'
		requestFormat: 'json'
		parameters: {
			path: { customer_id: string }

			body: Schemas.CreateCustomerContextRequest
		}
		responses: {
			201: {
				data: {
					address_line?: (string | null) | undefined
					city?: (string | null) | undefined
					created_at: string
					customer_id: Schemas.CustomerId
					id: Schemas.CustomerContextId
					label: string
					photo_key?: (string | null) | undefined
					postal_code?: (string | null) | undefined
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_GetEmployee = {
		method: 'GET'
		path: '/api/v1/employees/{employee_id}'
		requestFormat: 'json'
		parameters: {
			path: { employee_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EmployeeId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
					user_id?: (null | Schemas.UserId) | undefined
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteEmployee = {
		method: 'DELETE'
		path: '/api/v1/employees/{employee_id}'
		requestFormat: 'json'
		parameters: {
			path: { employee_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateEmployee = {
		method: 'PATCH'
		path: '/api/v1/employees/{employee_id}'
		requestFormat: 'json'
		parameters: {
			path: { employee_id: string }

			body: Schemas.UpdateEmployeeRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EmployeeId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
					user_id?: (null | Schemas.UserId) | undefined
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_GetEquipment = {
		method: 'GET'
		path: '/api/v1/equipment/{equipment_id}'
		requestFormat: 'json'
		parameters: {
			path: { equipment_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EquipmentId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteEquipment = {
		method: 'DELETE'
		path: '/api/v1/equipment/{equipment_id}'
		requestFormat: 'json'
		parameters: {
			path: { equipment_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateEquipment = {
		method: 'PATCH'
		path: '/api/v1/equipment/{equipment_id}'
		requestFormat: 'json'
		parameters: {
			path: { equipment_id: string }

			body: Schemas.UpdateEquipmentRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EquipmentId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type post_UploadFile = {
		method: 'POST'
		path: '/api/v1/files'
		requestFormat: 'binary'
		parameters: {
			body: Array<number>
		}
		responses: {
			201: {
				data: { key: string; mime_type: string; size_bytes: number }
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			413: unknown
			500: unknown
		}
	}
	export type get_ListOrganizations = {
		method: 'GET'
		path: '/api/v1/organizations'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					id: Schemas.OrganizationId
					name: string
					owner_id: Schemas.UserId
					slug: string
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
		}
	}
	export type post_CreateOrganization = {
		method: 'POST'
		path: '/api/v1/organizations'
		requestFormat: 'json'
		parameters: {
			body: Schemas.CreateOrganizationRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					id: Schemas.OrganizationId
					name: string
					owner_id: Schemas.UserId
					slug: string
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			409: unknown
		}
	}
	export type get_GetOrganization = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					id: Schemas.OrganizationId
					name: string
					owner_id: Schemas.UserId
					slug: string
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteOrganization = {
		method: 'DELETE'
		path: '/api/v1/organizations/{organization_id}'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateOrganization = {
		method: 'PATCH'
		path: '/api/v1/organizations/{organization_id}'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.UpdateOrganizationRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					id: Schemas.OrganizationId
					name: string
					owner_id: Schemas.UserId
					slug: string
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_ListCustomers = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/customers'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerId
					last_name: string
					organization_id: Schemas.OrganizationId
					phone?: (string | null) | undefined
					status: Schemas.CustomerStatus
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateCustomer = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/customers'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateCustomerRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					email?: (string | null) | undefined
					first_name: string
					id: Schemas.CustomerId
					last_name: string
					organization_id: Schemas.OrganizationId
					phone?: (string | null) | undefined
					status: Schemas.CustomerStatus
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_ListEmployees = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/employees'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EmployeeId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
					user_id?: (null | Schemas.UserId) | undefined
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateEmployee = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/employees'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateEmployeeRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EmployeeId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
					user_id?: (null | Schemas.UserId) | undefined
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_ListEquipment = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/equipment'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EquipmentId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateEquipment = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/equipment'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateEquipmentRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					hourly_rate_cents: number
					id: Schemas.EquipmentId
					name: string
					organization_id: Schemas.OrganizationId
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_ListProducts = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/products'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					description?: (string | null) | undefined
					id: Schemas.ProductId
					name: string
					organization_id: Schemas.OrganizationId
					sku?: (string | null) | undefined
					unit: Schemas.ServiceRateUnit
					unit_price_cents: number
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateProduct = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/products'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateProductRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					description?: (string | null) | undefined
					id: Schemas.ProductId
					name: string
					organization_id: Schemas.OrganizationId
					sku?: (string | null) | undefined
					unit: Schemas.ServiceRateUnit
					unit_price_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_ListQuotes = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/quotes'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					customer_context_id: Schemas.CustomerContextId
					customer_id: Schemas.CustomerId
					id: Schemas.QuoteId
					lines: Array<Schemas.QuoteLineResponse>
					organization_id: Schemas.OrganizationId
					reference: string
					status: Schemas.QuoteStatus
					title: string
					total_cents: number
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateQuote = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/quotes'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateQuoteRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					customer_context_id: Schemas.CustomerContextId
					customer_id: Schemas.CustomerId
					id: Schemas.QuoteId
					lines: Array<Schemas.QuoteLineResponse>
					organization_id: Schemas.OrganizationId
					reference: string
					status: Schemas.QuoteStatus
					title: string
					total_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_ListServiceRates = {
		method: 'GET'
		path: '/api/v1/organizations/{organization_id}/service-rates'
		requestFormat: 'json'
		parameters: {
			query: Partial<{ page: number; per_page: number }>
			path: { organization_id: string }
		}
		responses: {
			200: {
				data: Array<{
					created_at: string
					id: Schemas.ServiceRateId
					label: string
					organization_id: Schemas.OrganizationId
					rate_cents: number
					unit: Schemas.ServiceRateUnit
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
		}
	}
	export type post_CreateServiceRate = {
		method: 'POST'
		path: '/api/v1/organizations/{organization_id}/service-rates'
		requestFormat: 'json'
		parameters: {
			path: { organization_id: string }

			body: Schemas.CreateServiceRateRequest
		}
		responses: {
			201: {
				data: {
					created_at: string
					id: Schemas.ServiceRateId
					label: string
					organization_id: Schemas.OrganizationId
					rate_cents: number
					unit: Schemas.ServiceRateUnit
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			409: unknown
		}
	}
	export type get_GetProduct = {
		method: 'GET'
		path: '/api/v1/products/{product_id}'
		requestFormat: 'json'
		parameters: {
			path: { product_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					description?: (string | null) | undefined
					id: Schemas.ProductId
					name: string
					organization_id: Schemas.OrganizationId
					sku?: (string | null) | undefined
					unit: Schemas.ServiceRateUnit
					unit_price_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteProduct = {
		method: 'DELETE'
		path: '/api/v1/products/{product_id}'
		requestFormat: 'json'
		parameters: {
			path: { product_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateProduct = {
		method: 'PATCH'
		path: '/api/v1/products/{product_id}'
		requestFormat: 'json'
		parameters: {
			path: { product_id: string }

			body: Schemas.UpdateProductRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					description?: (string | null) | undefined
					id: Schemas.ProductId
					name: string
					organization_id: Schemas.OrganizationId
					sku?: (string | null) | undefined
					unit: Schemas.ServiceRateUnit
					unit_price_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_GetQuote = {
		method: 'GET'
		path: '/api/v1/quotes/{quote_id}'
		requestFormat: 'json'
		parameters: {
			path: { quote_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					customer_context_id: Schemas.CustomerContextId
					customer_id: Schemas.CustomerId
					id: Schemas.QuoteId
					lines: Array<Schemas.QuoteLineResponse>
					organization_id: Schemas.OrganizationId
					reference: string
					status: Schemas.QuoteStatus
					title: string
					total_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteQuote = {
		method: 'DELETE'
		path: '/api/v1/quotes/{quote_id}'
		requestFormat: 'json'
		parameters: {
			path: { quote_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateQuote = {
		method: 'PATCH'
		path: '/api/v1/quotes/{quote_id}'
		requestFormat: 'json'
		parameters: {
			path: { quote_id: string }

			body: Schemas.UpdateQuoteRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					customer_context_id: Schemas.CustomerContextId
					customer_id: Schemas.CustomerId
					id: Schemas.QuoteId
					lines: Array<Schemas.QuoteLineResponse>
					organization_id: Schemas.OrganizationId
					reference: string
					status: Schemas.QuoteStatus
					title: string
					total_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_ExportQuotePdf = {
		method: 'GET'
		path: '/api/v1/quotes/{quote_id}/pdf'
		requestFormat: 'json'
		parameters: {
			path: { quote_id: string }
		}
		responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateQuoteStatus = {
		method: 'PATCH'
		path: '/api/v1/quotes/{quote_id}/status'
		requestFormat: 'json'
		parameters: {
			path: { quote_id: string }

			body: Schemas.UpdateQuoteStatusRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					customer_context_id: Schemas.CustomerContextId
					customer_id: Schemas.CustomerId
					id: Schemas.QuoteId
					lines: Array<Schemas.QuoteLineResponse>
					organization_id: Schemas.OrganizationId
					reference: string
					status: Schemas.QuoteStatus
					title: string
					total_cents: number
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type get_GetServiceRate = {
		method: 'GET'
		path: '/api/v1/service-rates/{service_rate_id}'
		requestFormat: 'json'
		parameters: {
			path: { service_rate_id: string }
		}
		responses: {
			200: {
				data: {
					created_at: string
					id: Schemas.ServiceRateId
					label: string
					organization_id: Schemas.OrganizationId
					rate_cents: number
					unit: Schemas.ServiceRateUnit
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
			403: unknown
			404: unknown
		}
	}
	export type delete_DeleteServiceRate = {
		method: 'DELETE'
		path: '/api/v1/service-rates/{service_rate_id}'
		requestFormat: 'json'
		parameters: {
			path: { service_rate_id: string }
		}
		responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown }
	}
	export type patch_UpdateServiceRate = {
		method: 'PATCH'
		path: '/api/v1/service-rates/{service_rate_id}'
		requestFormat: 'json'
		parameters: {
			path: { service_rate_id: string }

			body: Schemas.UpdateServiceRateRequest
		}
		responses: {
			200: {
				data: {
					created_at: string
					id: Schemas.ServiceRateId
					label: string
					organization_id: Schemas.OrganizationId
					rate_cents: number
					unit: Schemas.ServiceRateUnit
					updated_at: string
				}
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			400: unknown
			401: unknown
			403: unknown
			404: unknown
			409: unknown
		}
	}
	export type get_ListMyOrganizations = {
		method: 'GET'
		path: '/api/v1/users/@me/organizations'
		requestFormat: 'json'
		parameters: never
		responses: {
			200: {
				data: Array<{
					created_at: string
					id: Schemas.OrganizationId
					name: string
					owner_id: Schemas.UserId
					slug: string
					updated_at: string
				}>
				pagination?: (null | Schemas.PaginationMetadata) | undefined
			}
			401: unknown
		}
	}

	// </Endpoints>
}

// <EndpointByMethod>
export type EndpointByMethod = {
	get: {
		'/api/v1/customer-contacts/{customer_contact_id}': Endpoints.get_GetCustomerContact
		'/api/v1/customer-contexts/{customer_context_id}': Endpoints.get_GetCustomerContext
		'/api/v1/customers/{customer_id}': Endpoints.get_GetCustomer
		'/api/v1/customers/{customer_id}/contacts': Endpoints.get_ListCustomerContacts
		'/api/v1/customers/{customer_id}/customer-contexts': Endpoints.get_ListCustomerContexts
		'/api/v1/employees/{employee_id}': Endpoints.get_GetEmployee
		'/api/v1/equipment/{equipment_id}': Endpoints.get_GetEquipment
		'/api/v1/organizations': Endpoints.get_ListOrganizations
		'/api/v1/organizations/{organization_id}': Endpoints.get_GetOrganization
		'/api/v1/organizations/{organization_id}/customers': Endpoints.get_ListCustomers
		'/api/v1/organizations/{organization_id}/employees': Endpoints.get_ListEmployees
		'/api/v1/organizations/{organization_id}/equipment': Endpoints.get_ListEquipment
		'/api/v1/organizations/{organization_id}/products': Endpoints.get_ListProducts
		'/api/v1/organizations/{organization_id}/quotes': Endpoints.get_ListQuotes
		'/api/v1/organizations/{organization_id}/service-rates': Endpoints.get_ListServiceRates
		'/api/v1/products/{product_id}': Endpoints.get_GetProduct
		'/api/v1/quotes/{quote_id}': Endpoints.get_GetQuote
		'/api/v1/quotes/{quote_id}/pdf': Endpoints.get_ExportQuotePdf
		'/api/v1/service-rates/{service_rate_id}': Endpoints.get_GetServiceRate
		'/api/v1/users/@me/organizations': Endpoints.get_ListMyOrganizations
	}
	delete: {
		'/api/v1/customer-contacts/{customer_contact_id}': Endpoints.delete_DeleteCustomerContact
		'/api/v1/customer-contexts/{customer_context_id}': Endpoints.delete_DeleteCustomerContext
		'/api/v1/customers/{customer_id}': Endpoints.delete_DeleteCustomer
		'/api/v1/employees/{employee_id}': Endpoints.delete_DeleteEmployee
		'/api/v1/equipment/{equipment_id}': Endpoints.delete_DeleteEquipment
		'/api/v1/organizations/{organization_id}': Endpoints.delete_DeleteOrganization
		'/api/v1/products/{product_id}': Endpoints.delete_DeleteProduct
		'/api/v1/quotes/{quote_id}': Endpoints.delete_DeleteQuote
		'/api/v1/service-rates/{service_rate_id}': Endpoints.delete_DeleteServiceRate
	}
	patch: {
		'/api/v1/customer-contacts/{customer_contact_id}': Endpoints.patch_UpdateCustomerContact
		'/api/v1/customer-contexts/{customer_context_id}': Endpoints.patch_UpdateCustomerContext
		'/api/v1/customers/{customer_id}': Endpoints.patch_UpdateCustomer
		'/api/v1/employees/{employee_id}': Endpoints.patch_UpdateEmployee
		'/api/v1/equipment/{equipment_id}': Endpoints.patch_UpdateEquipment
		'/api/v1/organizations/{organization_id}': Endpoints.patch_UpdateOrganization
		'/api/v1/products/{product_id}': Endpoints.patch_UpdateProduct
		'/api/v1/quotes/{quote_id}': Endpoints.patch_UpdateQuote
		'/api/v1/quotes/{quote_id}/status': Endpoints.patch_UpdateQuoteStatus
		'/api/v1/service-rates/{service_rate_id}': Endpoints.patch_UpdateServiceRate
	}
	post: {
		'/api/v1/customers/{customer_id}/contacts': Endpoints.post_CreateCustomerContact
		'/api/v1/customers/{customer_id}/customer-contexts': Endpoints.post_CreateCustomerContext
		'/api/v1/files': Endpoints.post_UploadFile
		'/api/v1/organizations': Endpoints.post_CreateOrganization
		'/api/v1/organizations/{organization_id}/customers': Endpoints.post_CreateCustomer
		'/api/v1/organizations/{organization_id}/employees': Endpoints.post_CreateEmployee
		'/api/v1/organizations/{organization_id}/equipment': Endpoints.post_CreateEquipment
		'/api/v1/organizations/{organization_id}/products': Endpoints.post_CreateProduct
		'/api/v1/organizations/{organization_id}/quotes': Endpoints.post_CreateQuote
		'/api/v1/organizations/{organization_id}/service-rates': Endpoints.post_CreateServiceRate
	}
}

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type GetEndpoints = EndpointByMethod['get']
export type DeleteEndpoints = EndpointByMethod['delete']
export type PatchEndpoints = EndpointByMethod['patch']
export type PostEndpoints = EndpointByMethod['post']
// </EndpointByMethod.Shorthands>

// <ApiClientTypes>
export type EndpointParameters = {
	body?: unknown
	query?: Record<string, unknown>
	header?: Record<string, unknown>
	path?: Record<string, unknown>
}

export type MutationMethod = 'post' | 'put' | 'patch' | 'delete'
export type Method = 'get' | 'head' | 'options' | MutationMethod

type RequestFormat = 'json' | 'form-data' | 'form-url' | 'binary' | 'text'

export type DefaultEndpoint = {
	parameters?: EndpointParameters | undefined
	responses?: Record<string, unknown>
	responseHeaders?: Record<string, unknown>
}

export type Endpoint<TConfig extends DefaultEndpoint = DefaultEndpoint> = {
	operationId: string
	method: Method
	path: string
	requestFormat: RequestFormat
	parameters?: TConfig['parameters']
	meta: {
		alias: string
		hasParameters: boolean
		areParametersRequired: boolean
	}
	responses?: TConfig['responses']
	responseHeaders?: TConfig['responseHeaders']
}

export interface Fetcher {
	decodePathParams?: (
		path: string,
		pathParams: Record<string, string>,
	) => string
	encodeSearchParams?: (
		searchParams: Record<string, unknown> | undefined,
	) => URLSearchParams
	//
	fetch: (input: {
		method: Method
		url: URL
		urlSearchParams?: URLSearchParams | undefined
		parameters?: EndpointParameters | undefined
		path: string
		overrides?: RequestInit
		throwOnStatusError?: boolean
	}) => Promise<Response>
	parseResponseData?: (response: Response) => Promise<unknown>
}

export const successStatusCodes = [
	200, 201, 202, 203, 204, 205, 206, 207, 208, 226, 300, 301, 302, 303, 304,
	305, 306, 307, 308,
] as const
export type SuccessStatusCode = (typeof successStatusCodes)[number]

export const errorStatusCodes = [
	400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414,
	415, 416, 417, 418, 421, 422, 423, 424, 425, 426, 428, 429, 431, 451, 500,
	501, 502, 503, 504, 505, 506, 507, 508, 510, 511,
] as const
export type ErrorStatusCode = (typeof errorStatusCodes)[number]

// Taken from https://github.com/unjs/fetchdts/blob/ec4eaeab5d287116171fc1efd61f4a1ad34e4609/src/fetch.ts#L3
export interface TypedHeaders<
	TypedHeaderValues extends Record<string, string> | unknown,
> extends Omit<
		Headers,
		'append' | 'delete' | 'get' | 'getSetCookie' | 'has' | 'set' | 'forEach'
	> {
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
	append: <
		Name extends Extract<keyof TypedHeaderValues, string> | (string & {}),
	>(
		name: Name,
		value: Lowercase<Name> extends keyof TypedHeaderValues
			? TypedHeaderValues[Lowercase<Name>]
			: string,
	) => void
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
	delete: <
		Name extends Extract<keyof TypedHeaderValues, string> | (string & {}),
	>(
		name: Name,
	) => void
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
	get: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
		name: Name,
	) =>
		| (Lowercase<Name> extends keyof TypedHeaderValues
				? TypedHeaderValues[Lowercase<Name>]
				: string)
		| null
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/getSetCookie) */
	getSetCookie: () => string[]
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
	has: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
		name: Name,
	) => boolean
	/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
	set: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
		name: Name,
		value: Lowercase<Name> extends keyof TypedHeaderValues
			? TypedHeaderValues[Lowercase<Name>]
			: string,
	) => void
	forEach: (
		callbackfn: (
			value: TypedHeaderValues[keyof TypedHeaderValues] | (string & {}),
			key: Extract<keyof TypedHeaderValues, string> | (string & {}),
			parent: TypedHeaders<TypedHeaderValues>,
		) => void,
		thisArg?: any,
	) => void
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedSuccessResponse<TSuccess, TStatusCode, THeaders>
	extends Omit<Response, 'ok' | 'status' | 'json' | 'headers'> {
	ok: true
	status: TStatusCode
	headers: never extends THeaders ? Headers : TypedHeaders<THeaders>
	data: TSuccess
	/** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
	json: () => Promise<TSuccess>
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedErrorResponse<TData, TStatusCode, THeaders>
	extends Omit<Response, 'ok' | 'status' | 'json' | 'headers'> {
	ok: false
	status: TStatusCode
	headers: never extends THeaders ? Headers : TypedHeaders<THeaders>
	data: TData
	/** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
	json: () => Promise<TData>
}

export type TypedApiResponse<
	TAllResponses extends Record<string | number, unknown> = {},
	THeaders = {},
> = {
	[K in keyof TAllResponses]: K extends string
		? K extends `${infer TStatusCode extends number}`
			? TStatusCode extends SuccessStatusCode
				? TypedSuccessResponse<
						TAllResponses[K],
						TStatusCode,
						K extends keyof THeaders ? THeaders[K] : never
					>
				: TypedErrorResponse<
						TAllResponses[K],
						TStatusCode,
						K extends keyof THeaders ? THeaders[K] : never
					>
			: never
		: K extends number
			? K extends SuccessStatusCode
				? TypedSuccessResponse<
						TAllResponses[K],
						K,
						K extends keyof THeaders ? THeaders[K] : never
					>
				: TypedErrorResponse<
						TAllResponses[K],
						K,
						K extends keyof THeaders ? THeaders[K] : never
					>
			: never
}[keyof TAllResponses]

export type SafeApiResponse<TEndpoint> = TEndpoint extends {
	responses: infer TResponses
}
	? TResponses extends Record<string, unknown>
		? TypedApiResponse<
				TResponses,
				TEndpoint extends { responseHeaders: infer THeaders } ? THeaders : never
			>
		: never
	: never

export type InferResponseByStatus<TEndpoint, TStatusCode> = Extract<
	SafeApiResponse<TEndpoint>,
	{ status: TStatusCode }
>

type RequiredKeys<T> = {
	[P in keyof T]-?: undefined extends T[P] ? never : P
}[keyof T]

type MaybeOptionalArg<T> =
	RequiredKeys<T> extends never ? [config?: T] : [config: T]
type NotNever<T> = [T] extends [never] ? false : true

// </ApiClientTypes>

// <TypedStatusError>
export class TypedStatusError<TData = unknown> extends Error {
	response: TypedErrorResponse<TData, ErrorStatusCode, unknown>
	status: number
	constructor(response: TypedErrorResponse<TData, ErrorStatusCode, unknown>) {
		super(`HTTP ${response.status}: ${response.statusText}`)
		this.name = 'TypedStatusError'
		this.response = response
		this.status = response.status
	}
}
// </TypedStatusError>

// <ApiClient>
export class ApiClient {
	baseUrl: string = ''
	successStatusCodes = successStatusCodes
	errorStatusCodes = errorStatusCodes

	constructor(public fetcher: Fetcher) {}

	setBaseUrl(baseUrl: string) {
		this.baseUrl = baseUrl
		return this
	}

	/**
	 * Replace path parameters in URL
	 * Supports both OpenAPI format {param} and Express format :param
	 */
	defaultDecodePathParams = (
		url: string,
		params: Record<string, string>,
	): string => {
		return url
			.replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
			.replace(
				/:([a-zA-Z0-9_]+)/g,
				(_, key: string) => params[key] || `:${key}`,
			)
	}

	/** Uses URLSearchParams, skips null/undefined values */
	defaultEncodeSearchParams = (
		queryParams: Record<string, unknown> | undefined,
	): URLSearchParams | undefined => {
		if (!queryParams) return

		const searchParams = new URLSearchParams()
		Object.entries(queryParams).forEach(([key, value]) => {
			if (value != null) {
				// Skip null/undefined values
				if (Array.isArray(value)) {
					value.forEach(
						(val) => val != null && searchParams.append(key, String(val)),
					)
				} else {
					searchParams.append(key, String(value))
				}
			}
		})

		return searchParams
	}

	defaultParseResponseData = async (response: Response): Promise<unknown> => {
		const contentType = response.headers.get('content-type') ?? ''
		if (contentType.startsWith('text/')) {
			return await response.text()
		}

		if (contentType === 'application/octet-stream') {
			return await response.arrayBuffer()
		}

		if (
			contentType.includes('application/json') ||
			(contentType.includes('application/') && contentType.includes('json')) ||
			contentType === '*/*'
		) {
			try {
				return await response.json()
			} catch {
				return undefined
			}
		}

		return
	}

	// <ApiClient.get>
	get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: false
						throwOnStatusError?: boolean
					}
		>
	): Promise<
		Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	>

	get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: true
						throwOnStatusError?: boolean
					}
		>
	): Promise<SafeApiResponse<TEndpoint>>

	get<Path extends keyof GetEndpoints, _TEndpoint extends GetEndpoints[Path]>(
		path: Path,
		...params: MaybeOptionalArg<any>
	): Promise<any> {
		return this.request('get', path, ...params)
	}
	// </ApiClient.get>

	// <ApiClient.delete>
	delete<
		Path extends keyof DeleteEndpoints,
		TEndpoint extends DeleteEndpoints[Path],
	>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: false
						throwOnStatusError?: boolean
					}
		>
	): Promise<
		Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	>

	delete<
		Path extends keyof DeleteEndpoints,
		TEndpoint extends DeleteEndpoints[Path],
	>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: true
						throwOnStatusError?: boolean
					}
		>
	): Promise<SafeApiResponse<TEndpoint>>

	delete<
		Path extends keyof DeleteEndpoints,
		_TEndpoint extends DeleteEndpoints[Path],
	>(path: Path, ...params: MaybeOptionalArg<any>): Promise<any> {
		return this.request('delete', path, ...params)
	}
	// </ApiClient.delete>

	// <ApiClient.patch>
	patch<
		Path extends keyof PatchEndpoints,
		TEndpoint extends PatchEndpoints[Path],
	>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: false
						throwOnStatusError?: boolean
					}
		>
	): Promise<
		Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	>

	patch<
		Path extends keyof PatchEndpoints,
		TEndpoint extends PatchEndpoints[Path],
	>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: true
						throwOnStatusError?: boolean
					}
		>
	): Promise<SafeApiResponse<TEndpoint>>

	patch<
		Path extends keyof PatchEndpoints,
		_TEndpoint extends PatchEndpoints[Path],
	>(path: Path, ...params: MaybeOptionalArg<any>): Promise<any> {
		return this.request('patch', path, ...params)
	}
	// </ApiClient.patch>

	// <ApiClient.post>
	post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: false
						throwOnStatusError?: boolean
					}
		>
	): Promise<
		Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	>

	post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
		path: Path,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: true
						throwOnStatusError?: boolean
					}
		>
	): Promise<SafeApiResponse<TEndpoint>>

	post<
		Path extends keyof PostEndpoints,
		_TEndpoint extends PostEndpoints[Path],
	>(path: Path, ...params: MaybeOptionalArg<any>): Promise<any> {
		return this.request('post', path, ...params)
	}
	// </ApiClient.post>

	// <ApiClient.request>
	/**
	 * Generic request method with full type-safety for any endpoint
	 */
	request<
		TMethod extends keyof EndpointByMethod,
		TPath extends keyof EndpointByMethod[TMethod],
		TEndpoint extends EndpointByMethod[TMethod][TPath],
	>(
		method: TMethod,
		path: TPath,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: false
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: false
						throwOnStatusError?: boolean
					}
		>
	): Promise<
		Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	>

	request<
		TMethod extends keyof EndpointByMethod,
		TPath extends keyof EndpointByMethod[TMethod],
		TEndpoint extends EndpointByMethod[TMethod][TPath],
	>(
		method: TMethod,
		path: TPath,
		...params: MaybeOptionalArg<
			TEndpoint extends { parameters: infer UParams }
				? NotNever<UParams> extends true
					? UParams & {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
					: {
							overrides?: RequestInit
							withResponse?: true
							throwOnStatusError?: boolean
						}
				: {
						overrides?: RequestInit
						withResponse?: true
						throwOnStatusError?: boolean
					}
		>
	): Promise<SafeApiResponse<TEndpoint>>

	request<
		TMethod extends keyof EndpointByMethod,
		TPath extends keyof EndpointByMethod[TMethod],
		TEndpoint extends EndpointByMethod[TMethod][TPath],
	>(
		method: TMethod,
		path: TPath,
		...params: MaybeOptionalArg<any>
	): Promise<any> {
		const requestParams = params[0]
		const withResponse = requestParams?.withResponse
		const {
			withResponse: _,
			throwOnStatusError = withResponse ? false : true,
			overrides,
			...fetchParams
		} = requestParams || {}

		const parametersToSend: EndpointParameters = {}
		if (requestParams?.body !== undefined)
			(parametersToSend as any).body = requestParams.body
		if (requestParams?.query !== undefined)
			(parametersToSend as any).query = requestParams.query
		if (requestParams?.header !== undefined)
			(parametersToSend as any).header = requestParams.header
		if (requestParams?.path !== undefined)
			(parametersToSend as any).path = requestParams.path

		const resolvedPath = (
			this.fetcher.decodePathParams ?? this.defaultDecodePathParams
		)(
			this.baseUrl + (path as string),
			(parametersToSend.path ?? {}) as Record<string, string>,
		)
		const url = new URL(resolvedPath)
		const urlSearchParams = (
			this.fetcher.encodeSearchParams ?? this.defaultEncodeSearchParams
		)(parametersToSend.query)

		const promise = this.fetcher
			.fetch({
				method: method,
				path: path as string,
				url,
				urlSearchParams,
				parameters: Object.keys(fetchParams).length ? fetchParams : undefined,
				overrides,
				throwOnStatusError,
			})
			.then(async (response) => {
				const data = await (
					this.fetcher.parseResponseData ?? this.defaultParseResponseData
				)(response)
				const typedResponse = Object.assign(response, {
					data: data,
					json: () => Promise.resolve(data),
				}) as SafeApiResponse<TEndpoint>

				if (
					throwOnStatusError &&
					errorStatusCodes.includes(response.status as never)
				) {
					throw new TypedStatusError(typedResponse as never)
				}

				return withResponse ? typedResponse : data
			})

		return promise as Extract<
			InferResponseByStatus<TEndpoint, SuccessStatusCode>,
			{ data: {} }
		>['data']
	}
	// </ApiClient.request>
}

export function createApiClient(fetcher: Fetcher, baseUrl?: string) {
	return new ApiClient(fetcher).setBaseUrl(baseUrl ?? '')
}

/**
 Example usage:
 const api = createApiClient((method, url, params) =>
   fetch(url, { method, body: JSON.stringify(params) }).then((res) => res.json()),
 );
 api.get("/users").then((users) => console.log(users));
 api.post("/users", { body: { name: "John" } }).then((user) => console.log(user));
 api.put("/users/:id", { path: { id: 1 }, body: { name: "John" } }).then((user) => console.log(user));

 // With error handling
 const result = await api.get("/users/{id}", { path: { id: "123" }, withResponse: true });
 if (result.ok) {
   // Access data directly
   const user = result.data;
   console.log(user);

   // Or use the json() method for compatibility
   const userFromJson = await result.json();
   console.log(userFromJson);
 } else {
   const error = result.data;
   console.error(`Error ${result.status}:`, error);
 }
*/

// </ApiClient>

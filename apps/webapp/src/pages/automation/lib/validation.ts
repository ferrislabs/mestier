import type { Schemas } from '#/api/api.client'

export interface ConnectorValidationError {
	field: string | null
	message: string
}

export interface GraphValidation {
	connectorErrors: Map<string, ConnectorValidationError[]>
	graphErrors: string[]
}

export function projectGraphErrors(
	errors: Schemas.GraphErrorResponse[],
): GraphValidation {
	const connectorErrors = new Map<string, ConnectorValidationError[]>()
	const graphErrors: string[] = []

	for (const error of errors) {
		const nodeId = error.connector_id ?? error.trigger_id
		if (!nodeId) {
			graphErrors.push(error.message)
			continue
		}

		const existing = connectorErrors.get(nodeId) ?? []
		existing.push({ field: error.field ?? null, message: error.message })
		connectorErrors.set(nodeId, existing)
	}

	return { connectorErrors, graphErrors }
}

export function fieldErrorMessage(
	errors: ConnectorValidationError[],
	field: string,
): string | null {
	return errors.find((error) => error.field === field)?.message ?? null
}

export function connectorLevelErrors(
	errors: ConnectorValidationError[],
): string[] {
	return errors
		.filter((error) => error.field === null)
		.map((error) => error.message)
}

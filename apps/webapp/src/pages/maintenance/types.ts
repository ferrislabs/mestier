import type { Schemas } from '#/api/api.client'

export type Grid = { year: number; contracts: Schemas.GridContract[] }
export type GridContract = Schemas.GridContract
export type GridLine = Schemas.GridLine
export type GridVisit = Schemas.GridVisit
export type Season = Schemas.Season
export type Cadence = Schemas.Cadence
export type ContractStatus = Schemas.ContractStatus
export type MaintenanceContract = Schemas.MaintenanceContractResponse

/** The sheet of the paper original: S12 to S48. */
export const DEFAULT_FIRST_WEEK = 12
export const DEFAULT_LAST_WEEK = 48

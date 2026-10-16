import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import {
	useArchiveMaintenanceContract,
	useCreateMaintenanceContract,
	useDuplicateMaintenanceContract,
	useMaintenanceContracts,
	useSitesOfCustomers,
	useUpdateMaintenanceContract,
} from '#/hooks/use-maintenance-contracts'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	activationBody,
	type ContractDraft,
	contractIssues,
	contractToDraft,
	emptyContractDraft,
	emptyLineDraft,
	hasIssues,
	type LineDraft,
	toContractBody,
	toCreateRequest,
} from '#/pages/maintenance/lib/contract-form'
import { splitDrafts } from '#/pages/maintenance/lib/contract-list'
import type { MaintenanceContract } from '#/pages/maintenance/types'
import {
	type ContractRow,
	ContractsUi,
} from '#/pages/maintenance/ui/contracts-ui'

export interface ContractsFeatureProps {
	/** The season's year, from the address. */
	season: number
	onSeasonChange: (season: number) => void
}

export function ContractsFeature(props: ContractsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ContractsScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			{...props}
		/>
	)
}

function ContractsScreen({
	organizationId,
	organizationName,
	season,
	onSeasonChange,
}: ContractsFeatureProps & {
	organizationId: string
	organizationName: string
}) {
	const [includeArchived, setIncludeArchived] = useState(false)
	const [draft, setDraft] = useState<ContractDraft | null>(null)
	const [showIssues, setShowIssues] = useState(false)
	const [formError, setFormError] = useState<string | null>(null)
	const [archiveTarget, setArchiveTarget] = useState<ContractRow | null>(null)
	const [activateTarget, setActivateTarget] = useState<ContractRow | null>(null)
	const [notice, setNotice] = useState<{
		message: string
		season: number
	} | null>(null)
	const [actionError, setActionError] = useState<string | null>(null)

	const contractsQuery = useMaintenanceContracts(
		organizationId,
		season,
		includeArchived,
	)
	const customersQuery = useCustomers(organizationId)
	const members = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	}).members
	const draftSites = useCustomerContexts(
		draft?.customerId ?? '',
		Boolean(draft?.customerId),
	)

	const create = useCreateMaintenanceContract()
	const update = useUpdateMaintenanceContract()
	const activate = useUpdateMaintenanceContract()
	const archive = useArchiveMaintenanceContract()
	const duplicate = useDuplicateMaintenanceContract()

	const contracts = contractsQuery.data?.data ?? []
	const customers = customersQuery.data?.data ?? []
	const memberRows = members.data?.data ?? []

	const customerIds = useMemo(
		() => [...new Set(contracts.map((contract) => contract.customer_id))],
		[contracts],
	)
	const siteQueries = useSitesOfCustomers(customerIds)

	const rows: ContractRow[] = useMemo(() => {
		const customerName = new Map(customers.map((c) => [c.id, c.name]))
		const siteLabel = new Map<string, string>()
		for (const query of siteQueries) {
			for (const site of query.data?.data ?? [])
				siteLabel.set(site.id, site.label)
		}
		const managerName = new Map(memberRows.map((m) => [m.id, m.display_name]))

		return contracts.map((contract) => ({
			contract,
			customerName: customerName.get(contract.customer_id) ?? null,
			siteLabel: siteLabel.get(contract.customer_context_id) ?? null,
			managerName: contract.manager_member_id
				? (managerName.get(contract.manager_member_id) ?? null)
				: null,
		}))
	}, [contracts, customers, memberRows, siteQueries])

	const { drafts: draftRows, active: activeRows } = useMemo(
		() => splitDrafts(rows),
		[rows],
	)

	const issues = draft ? contractIssues(draft) : { lines: {} }
	const isSaving = create.isPending || update.isPending

	function openForm(next: ContractDraft) {
		setDraft(next)
		setShowIssues(false)
		setFormError(null)
		create.reset()
		update.reset()
	}

	function closeForm() {
		setDraft(null)
		setShowIssues(false)
		setFormError(null)
	}

	function changeLine(key: string, patch: Partial<LineDraft>) {
		setDraft((current) =>
			current
				? {
						...current,
						lines: current.lines.map((line) =>
							line.key === key ? { ...line, ...patch } : line,
						),
					}
				: current,
		)
	}

	function save() {
		if (!draft) return
		if (hasIssues(contractIssues(draft))) {
			setShowIssues(true)
			return
		}
		setFormError(null)
		const onError = (failure: unknown) =>
			setFormError(mutationErrorMessage(failure))

		if (draft.id === null) {
			create.mutate(
				{
					path: { organization_id: organizationId },
					body: toCreateRequest(draft),
				},
				{
					onSuccess: () => {
						// The new contract is in its own year: follow it there.
						closeForm()
						onSeasonChange(Number(draft.year))
					},
					onError,
				},
			)
			return
		}
		update.mutate(
			{
				path: { organization_id: organizationId, contract_id: draft.id },
				body: toContractBody(draft),
			},
			{ onSuccess: closeForm, onError },
		)
	}

	function duplicateContract(contract: MaintenanceContract) {
		setActionError(null)
		setNotice(null)
		duplicate.mutate(
			{
				path: {
					organization_id: organizationId,
					contract_id: contract.id,
				},
			},
			{
				onSuccess: (response) =>
					setNotice({
						message: `Contrat dupliqué pour ${response.data.season.year}, sans intervention.`,
						season: response.data.season.year,
					}),
				onError: (failure) => setActionError(mutationErrorMessage(failure)),
			},
		)
	}

	function confirmArchive() {
		if (!archiveTarget) return
		const id = archiveTarget.contract.id
		setArchiveTarget(null)
		setActionError(null)
		archive.mutate(
			{ path: { organization_id: organizationId, contract_id: id } },
			{ onError: (failure) => setActionError(mutationErrorMessage(failure)) },
		)
	}

	function confirmActivate() {
		if (!activateTarget) return
		const { contract } = activateTarget
		setActivateTarget(null)
		setActionError(null)
		setNotice(null)
		// The grid and the recap read the contract's status: the hook refreshes
		// every maintenance query once the PUT has gone through.
		activate.mutate(
			{
				path: { organization_id: organizationId, contract_id: contract.id },
				body: activationBody(contract),
			},
			{ onError: (failure) => setActionError(mutationErrorMessage(failure)) },
		)
	}

	const pendingContractId =
		(activate.isPending && activate.variables?.path.contract_id) ||
		(duplicate.isPending && duplicate.variables?.path.contract_id) ||
		(archive.isPending && archive.variables?.path.contract_id) ||
		null

	return (
		<ContractsUi
			organizationName={organizationName}
			season={season}
			rows={activeRows}
			draftRows={draftRows}
			includeArchived={includeArchived}
			isLoading={contractsQuery.isLoading}
			error={
				contractsQuery.isError
					? mutationErrorMessage(contractsQuery.error)
					: actionError
			}
			notice={notice}
			pendingContractId={pendingContractId}
			archiveTarget={archiveTarget}
			activateTarget={activateTarget}
			form={{
				draft,
				customers: customers.map((c) => ({ id: c.id, label: c.name })),
				sites: (draftSites.data?.data ?? []).map((site) => ({
					id: site.id,
					label: site.label,
				})),
				isLoadingSites: draftSites.isLoading,
				managers: memberRows.map((m) => ({ id: m.id, label: m.display_name })),
				issues,
				showIssues,
				isPending: isSaving,
				error: formError,
				onChange: (patch) =>
					setDraft((current) => (current ? { ...current, ...patch } : current)),
				onLineChange: changeLine,
				onAddLine: () =>
					setDraft((current) =>
						current
							? { ...current, lines: [...current.lines, emptyLineDraft()] }
							: current,
					),
				onRemoveLine: (key) =>
					setDraft((current) =>
						current
							? {
									...current,
									lines: current.lines.filter((line) => line.key !== key),
								}
							: current,
					),
				onSubmit: save,
				onClose: closeForm,
			}}
			onSeasonChange={onSeasonChange}
			onIncludeArchivedChange={setIncludeArchived}
			onCreate={() => openForm(emptyContractDraft(season))}
			onEdit={(contract) => openForm(contractToDraft(contract))}
			onDuplicate={duplicateContract}
			onAskArchive={setArchiveTarget}
			onConfirmArchive={confirmArchive}
			onAskActivate={setActivateTarget}
			onConfirmActivate={confirmActivate}
			onDismissNotice={() => setNotice(null)}
		/>
	)
}

import { useState } from 'react'
import { PageShell } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useClosureReasons,
	useHostCompanies,
	useOrganizationMembers,
	useParticipantDetails,
} from '#/pages/cursus/shared/feature/cursus-queries'
import { todayIso } from '#/pages/cursus/shared/lib/format'
import {
	companyTutorOptions,
	type SelectOption,
} from '#/pages/cursus/shared/lib/options'
import { CloseDialog } from '#/pages/cursus/shared/ui/close-dialog'
import {
	type Contract,
	useAttachContractFile,
	useCloseContract,
	useCloseTriad,
	useCreateContract,
	useOpenContractFile,
	useSetContractTutors,
	useTriad,
	useUpdateContract,
	useUpdateTriad,
} from '#/pages/cursus/triads/feature/use-triads'
import {
	contractToForm,
	EMPTY_CONTRACT_FORM,
	toContractRequest,
} from '#/pages/cursus/triads/lib/contract-form'
import { isClosed } from '#/pages/cursus/triads/lib/triad-form'
import { ContractFormDialog } from '#/pages/cursus/triads/ui/contract-form-dialog'
import { ContractTutorsDialog } from '#/pages/cursus/triads/ui/contract-tutors-dialog'
import { ContractsSection } from '#/pages/cursus/triads/ui/contracts-section'
import { TriadHeader } from '#/pages/cursus/triads/ui/triad-header'

type Dialogs =
	| { kind: 'none' }
	| { kind: 'add-contract' }
	| { kind: 'edit-contract'; contract: Contract }
	| { kind: 'tutors'; contract: Contract }
	| { kind: 'close-contract'; contract: Contract }
	| { kind: 'close-triad' }

export interface TriadDetailFeatureProps {
	triadId: string
	onBack: () => void
	onOpenPromotion: (promotionId: string) => void
}

export function TriadDetailFeature({
	triadId,
	onBack,
	onOpenPromotion,
}: TriadDetailFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [dialog, setDialog] = useState<Dialogs>({ kind: 'none' })
	const triadQuery = useTriad(activeOrganizationId, triadId)
	const triad = triadQuery.data?.data

	const members = useOrganizationMembers(activeOrganizationId, canManage)
	const hostCompanies = useHostCompanies(
		activeOrganizationId,
		canManage &&
			(dialog.kind === 'add-contract' || dialog.kind === 'edit-contract'),
	)
	const reasons = useClosureReasons(
		activeOrganizationId,
		canManage &&
			(dialog.kind === 'close-contract' || dialog.kind === 'close-triad'),
	)
	const tutorsOpen = dialog.kind === 'tutors'
	const tutorParticipants = useParticipantDetails(
		activeOrganizationId,
		'company_tutor',
		tutorsOpen,
	)

	const updateTriad = useUpdateTriad()
	const closeTriad = useCloseTriad()
	const createContract = useCreateContract()
	const updateContract = useUpdateContract()
	const closeContract = useCloseContract()
	const setTutors = useSetContractTutors()
	const files = useAttachContractFile(activeOrganizationId)
	const openFile = useOpenContractFile(activeOrganizationId)

	if (triadQuery.isError) {
		return (
			<PageShell>
				<p role="alert" className="text-state-alert">
					Trinôme introuvable.
				</p>
			</PageShell>
		)
	}
	if (!triad) {
		return (
			<PageShell>
				<p className="text-muted-foreground">Chargement…</p>
			</PageShell>
		)
	}

	const memberOptions: SelectOption[] = members.map((member) => ({
		value: member.id,
		label: member.display_name,
	}))
	const reasonOptions: SelectOption[] = (reasons.data?.data ?? []).map(
		(reason) => ({ value: reason.id, label: reason.label }),
	)
	const currentCompany =
		dialog.kind === 'edit-contract'
			? {
					value: dialog.contract.host_company_id,
					label: dialog.contract.host_company_name,
				}
			: null
	const companyOptions: SelectOption[] = (hostCompanies.data?.data ?? [])
		.filter((company) => !company.archived_at)
		.map((company) => ({ value: company.id, label: company.name }))
	if (
		currentCompany &&
		!companyOptions.some((option) => option.value === currentCompany.value)
	) {
		companyOptions.push(currentCompany)
	}
	const closeDialog = () => setDialog({ kind: 'none' })
	const org = activeOrganizationId
	const triadClosed = isClosed(triad)
	const contractMain = (contract: Contract) =>
		contract.tutors.find((tutor) => tutor.rank === 'main' && !tutor.archived_at)
			?.company_tutor_id ?? null
	const contractSecondary = (contract: Contract) =>
		contract.tutors.find(
			(tutor) => tutor.rank === 'secondary' && !tutor.archived_at,
		)?.company_tutor_id ?? null

	return (
		<PageShell>
			<TriadHeader
				triad={triad}
				members={memberOptions}
				canManage={canManage}
				isUpdatingTutor={updateTriad.isPending}
				tutorError={mutationErrorMessage(updateTriad.error)}
				onBack={onBack}
				onOpenPromotion={() => onOpenPromotion(triad.promotion_id)}
				onTutorChange={(memberId) =>
					updateTriad.mutate({
						path: { organization_id: org, triad_id: triad.id },
						body: { pedagogic_tutor_member_id: memberId },
					})
				}
				onClose={() => setDialog({ kind: 'close-triad' })}
			/>
			<ContractsSection
				contracts={triad.contracts}
				canManage={canManage && !triadClosed}
				canAdd={!triadClosed}
				isUploading={files.isPending}
				fileError={mutationErrorMessage(files.error)}
				onAdd={() => setDialog({ kind: 'add-contract' })}
				onEdit={(contract) => setDialog({ kind: 'edit-contract', contract })}
				onTutors={(contract) => setDialog({ kind: 'tutors', contract })}
				onClose={(contract) => setDialog({ kind: 'close-contract', contract })}
				onOpenFile={(contract, kind) => void openFile(contract.id, kind)}
				onUploadFile={(contract, kind, file) =>
					void files.attach(contract.id, kind, file).catch(() => undefined)
				}
			/>

			<ContractFormDialog
				key={dialog.kind === 'edit-contract' ? dialog.contract.id : 'new'}
				open={dialog.kind === 'add-contract' || dialog.kind === 'edit-contract'}
				title={
					dialog.kind === 'edit-contract'
						? 'Modifier le contrat'
						: 'Ajouter un contrat'
				}
				submitLabel={
					dialog.kind === 'edit-contract' ? 'Enregistrer' : 'Ajouter le contrat'
				}
				initial={
					dialog.kind === 'edit-contract'
						? contractToForm(dialog.contract)
						: EMPTY_CONTRACT_FORM
				}
				hostCompanies={companyOptions}
				isPending={createContract.isPending || updateContract.isPending}
				error={mutationErrorMessage(
					dialog.kind === 'edit-contract'
						? updateContract.error
						: createContract.error,
				)}
				onOpenChange={(next) => {
					if (!next) {
						createContract.reset()
						updateContract.reset()
						closeDialog()
					}
				}}
				onSubmit={(values) => {
					if (dialog.kind === 'edit-contract') {
						updateContract.mutate(
							{
								path: { organization_id: org, contract_id: dialog.contract.id },
								body: toContractRequest(values),
							},
							{ onSuccess: closeDialog },
						)
						return
					}
					createContract.mutate(
						{
							path: { organization_id: org, triad_id: triad.id },
							body: toContractRequest(values),
						},
						{ onSuccess: closeDialog },
					)
				}}
			/>

			{dialog.kind === 'tutors' ? (
				<ContractTutorsDialog
					open
					hostCompanyName={dialog.contract.host_company_name}
					initial={{
						main: contractMain(dialog.contract),
						secondary: contractSecondary(dialog.contract),
					}}
					tutors={companyTutorOptions(
						tutorParticipants.details,
						dialog.contract.host_company_id,
					)}
					isLoadingOptions={tutorParticipants.isLoading}
					isPending={setTutors.isPending}
					error={mutationErrorMessage(setTutors.error)}
					onOpenChange={(next) => {
						if (!next) {
							setTutors.reset()
							closeDialog()
						}
					}}
					onSubmit={(values) =>
						setTutors.mutate(
							{
								path: { organization_id: org, contract_id: dialog.contract.id },
								body: { main: values.main, secondary: values.secondary },
							},
							{ onSuccess: closeDialog },
						)
					}
				/>
			) : null}

			<CloseDialog
				open={dialog.kind === 'close-contract'}
				title="Clôturer le contrat"
				description="Le contrat devient en lecture seule et ses tuteurs sont détachés."
				submitLabel="Clôturer le contrat"
				reasons={reasonOptions}
				initialDate={todayIso()}
				isPending={closeContract.isPending}
				error={mutationErrorMessage(closeContract.error)}
				onOpenChange={(next) => {
					if (!next) {
						closeContract.reset()
						closeDialog()
					}
				}}
				onSubmit={(values) => {
					if (dialog.kind !== 'close-contract') return
					closeContract.mutate(
						{
							path: { organization_id: org, contract_id: dialog.contract.id },
							body: {
								closed_at: values.closedAt,
								closure_reason_id: values.closureReasonId,
								closure_note: values.closureNote.trim() || null,
							},
						},
						{ onSuccess: closeDialog },
					)
				}}
			/>

			<CloseDialog
				open={dialog.kind === 'close-triad'}
				title="Clôturer le trinôme"
				description="Les contrats encore ouverts sont clôturés avec le trinôme."
				submitLabel="Clôturer le trinôme"
				reasons={reasonOptions}
				initialDate={todayIso()}
				isPending={closeTriad.isPending}
				error={mutationErrorMessage(closeTriad.error)}
				onOpenChange={(next) => {
					if (!next) {
						closeTriad.reset()
						closeDialog()
					}
				}}
				onSubmit={(values) =>
					closeTriad.mutate(
						{
							path: { organization_id: org, triad_id: triad.id },
							body: {
								closed_at: values.closedAt,
								closure_reason_id: values.closureReasonId,
								closure_note: values.closureNote.trim() || null,
							},
						},
						{ onSuccess: closeDialog },
					)
				}
			/>
		</PageShell>
	)
}

# 0015. Cursus participants are a separate kind of access, not organization members

Status: accepted
Date: 2026-11-09

## Context
The Cursus module replaces Studea, an apprenticeship tracking tool, for training centres such as Ascor. Apprentices and company tutors must log in to fill their booklet. They are external to the organization. Until now, the only way into an organization was a `Member` seat, and permission bits apply organization-wide.

## Decision
External people are `cursus.participant` rows: an organization-scoped person carrying an optional FerrisKey `user_id`, filled by a module-specific invitation. A participant is never a `Member`. Their access is computed per request from the triads they belong to (as apprentice, or as tutor through a contract), never from permission bits. They use a separate front space, `/portail`, outside `OrgGate`. One FerrisKey identity covers both kinds of access: a member of organization A may be a participant in organization B. In the organization where someone is a member, they always use the console, and their own triads appear under "Mes trinômes".

## Alternatives rejected
- Members with a restricted role: every other part of Mestier (member list, channels, organization switcher) would have to exclude them one by one, forever.
- Personal records carrying `user_id` directly (`Apprentice.user_id`, `CompanyTutor.user_id`): resolving "who is this external user here" would read one table per participant kind.
- A separate FerrisKey realm for participants: two identities for an artisan who already uses Mestier.

## Consequences
- Gains: structural isolation. A participant reaches nothing outside the Cursus portal routes, because they are not a member.
- Costs: a second authorization path (`ParticipantScope`) that every portal query must apply, and a second invitation flow.
- What becomes harder: a future customer or supplier portal must decide whether to reuse `participant` or build its own; this ADR does not settle it.

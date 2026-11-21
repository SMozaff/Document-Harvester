# Stage S1 — New Project owner acceptance

**Status: Blocked: manual acceptance pending.** Run this check against the Linux or Windows release artifact produced by the Stage S1 CI workflow. Do not record any private application-data path or workbook path in the result.

## Test procedure

1. Download the `s1-build-Linux` or `s1-build-Windows` artifact from the PR's successful CI run and note its commit SHA.
2. Launch `dh-app` (Linux) or `dh-app.exe` (Windows) with no command-line arguments.
3. Choose a writable parent folder, enter a new project name, and select **Create Project**.
4. Confirm the UI reports successful creation and shows the workbook location.
5. Open the generated `.xlsx` in LibreOffice Calc or Microsoft Excel. Confirm it opens without a repair/corruption warning and contains the intended empty destination worksheet, `Sheet1`, with no captured data.
6. Return to Document Harvester and choose **Create another project**. Create a second project with a different name. Open its workbook and confirm it is a separate empty workbook; confirm the first workbook remains unchanged. This checks that the projects do not share a workbook.
7. Close Document Harvester completely and launch it again with no arguments.
8. Confirm the most recently created project is restored in the application and its workbook association is still present. Confirm the first workbook remains separate and unchanged.
9. Record the result below. If any step fails, mark Fail and describe the symptom without including private paths.

## Owner result

| Field | Owner entry |
|---|---|
| OS and version | |
| Build commit SHA | |
| Spreadsheet application and version | |
| Result (Pass/Fail) | |
| Notes (no private paths) | |

**Acceptance rule:** This stage remains **Blocked: manual acceptance pending** until the owner fills in the result and reports Pass. Do not mark Stage 1 complete based only on CI.

# Uninstall and restore a dojo

In Keiko, open a dojo’s **…** menu, choose **Move to Trash…**, then confirm. Cancel or Escape leaves the app installed. Close the dojo first if you do not want its current process to keep running.

Keiko moves the `.app` bundle to the Trash. It keeps saved drafts, lesson completion, profiles, settings and other dojo data in Application Support. For PodDojo, removing the app does not remove Docker resources, the lab cluster or its tools. Cluster cleanup is a separate explicit action in PodDojo; it never happens as part of app uninstall.

To undo, restore the app from the Trash to its original Applications folder. Bring Keiko back to the front or refresh; the installed card returns. You can also choose Install from Keiko to download the app again; retained learning data remains available.

Keiko refuses a source that is a link/file or whose bundle identifier does not match the catalog. It reserves an unused destination such as `TypeDojo (1).app` so another trashed app is never overwritten. If the move cannot be completed, it keeps the installed bundle and shows an error. Cross-volume moves require Finder; Keiko does not copy and delete the source as a fallback.

## Verify safely as a contributor

Set **both** `KEIKO_HOME` and `KEIKO_APPS_DIR` to dedicated temporary directories. Detection/install only use the isolated app directory; uninstallation uses `$KEIKO_HOME/Trash`, not the learner’s real `~/.Trash`. The normal app uses the system Trash. `Paths::for_tests` also supplies a private Trash for native tests.

Native unit cases cover restore/data preservation, occupied bundle/file/dangling-link names, mismatched identifiers/source links and failures that must keep the app. The ignored live-network tests use real signed PodDojo/TypeDojo releases for install → checksum refusal → uninstall → restore and verify the signature and data preservation.

Review the native confirmation dialog, Cancel, confirm, Available/Install and reinstall using copies in those isolated folders. Never use a learner’s real installed apps as test fixtures.

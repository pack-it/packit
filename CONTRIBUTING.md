# Contributing to Packit

Thank you for considering to contribute to Packit! This file will help you in understanding how to contribute to Packit.

If you want to add support for a new package, a new package version or some other change to the list of available packages, please take a look at our [core repository](https://github.com/pack-it/core).

## Getting started

The steps below guide you through the process of forking the repository, commiting changes and opening a pull request.

1. **Fork the repository** <br>
Click the `Fork` button on the top-right corner of the repository page. This creates your own copy of the Packit repository, where you can make your changes.

2. **Clone your fork locally** <br>
To start working, you need the repository on your local machine. To do this, you need to clone it.
<br><br>
Run the commands below to clone your fork locally, replace `<your-username>` with your GitHub username.
<br><br>
```
git clone https://github.com/<your-username>/packit.git
cd packit
```

3. **Create a new branch** <br>
Now you have a local copy of the repository, you need to create a new branch where you can make your changes.
<br><br>
Please use a name which describes your changes for the branch.
<br><br>
```
git checkout -b <branch-name>
```

4. **Add your changes** <br>
On the new branch, you can now add your own changes.

5. **Pushing your changes** <br>
After you added your changes, you first need to create a commit and push your changes.
<br><br>
Please ensure your commit message describes the changes correctly
<br><br>
You can add, commit and push by running the command below:
```
git add packages/<package-name>
git commit -m <message>
git push origin <branch-name>
```

6. **Open a pull request** <br>
Now all your changes are on your own fork of the Packit repository, you can open a pull request to submit your changes to the official Packit repository.
<br><br>
To create a pull request you need to go to the official repository (https://github.com/pack-it/packit), at the top a yellow banner is shown with the button `Compare & pull request`. Click on this button and ensure the `base` branch is set to `main` and the `compare` branch is set to the branch you created.
<br><br>
Ensure the titles describes clearly what changes you made, ideally this matches with the commit naming conventions. Add a good description, including all details of your changes.
<br><br>
After you done all steps above, click on the `Create pull request` button. Your pull request is now created and one of the Packit maintainers will review it as soon as possible.

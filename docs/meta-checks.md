# Metadata checks

The metadata checks perform checks on the metadata. It looks for missing, inconsistent or incomplete data. The issues are collected and then shown at the end of the checks. The following table shows the different issue types and what they mean.
| Issue Type  | Explanation                                                                                          |
| ----------- | ---------------------------------------------------------------------------------------------------- |
| `Fatal`     | The issue is so severe that the checks cannot be continued and the command has to exit early.        |
| `Breaking`  | The metadata checks have found an issue which will break certain logic, but continueing is possible. |
| `Warning`   | The metadata is correct and functions, however it's unconventional (although maybe unavoidable)      |

In certain cases it's possible to make a good guess about what would be a correct metadata value. In such a this value is shown as a suggestion together with the issue. Occasionally some checks are skipped due to a found issue, if this happens it is shown to you together with the issue.

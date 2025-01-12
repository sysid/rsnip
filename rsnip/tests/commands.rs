// commands.rs

use anyhow::Result;

use assert_cmd::Command;

#[test]
fn given_types_list_flag_when_execute_then_outputs_space_separated() -> Result<()> {
    // Arrange (run the actual binary)
    let mut cmd = Command::cargo_bin("rsnip")?;

    // Act
    let output = cmd.arg("types")
                   .arg("--list")
                   .assert()
                   .success();

    // Assert
    // todo: make it hermetic
    output.stdout(predicates::str::contains("general shell"));

    Ok(())
}

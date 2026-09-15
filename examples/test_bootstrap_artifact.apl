# Test APL bootstrap artifact facade.

AVStr ok_source = "AVInt x = 1 out x"
AVStr bad_source = "AVInt x 1 out x"

VTime ok_report = bootstrap.artifact_report(ok_source)
VTime bad_report = bootstrap.artifact_report(bad_source)
VTime artifact = get(ok_report, 1)

out "Bootstrap artifact:"
out get(ok_report, 0)
out artifact[:7]
out contains(artifact, "DECL")
out contains(artifact, "OUT")
out get(bad_report, 0)
out get(bad_report, 1)
out "Done!"

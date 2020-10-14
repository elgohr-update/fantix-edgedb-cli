use test_case::test_case;

use crate::docker::{Context, build_image, run, run_with_socket};
use crate::common::{dock_ubuntu, dock_centos, dock_debian};



#[test_case("edbtest_bionic", &dock_ubuntu("bionic"))]
#[test_case("edbtest_xenial", &dock_ubuntu("xenial"))]
#[test_case("edbtest_focal", &dock_ubuntu("focal"))]
#[test_case("edbtest_centos7", &dock_centos(7))]
#[test_case("edbtest_centos8", &dock_centos(8))]
#[test_case("edbtest_buster", &dock_debian("buster"))]
#[test_case("edbtest_stretch", &dock_debian("stretch"))]
fn package(tagname: &str, dockerfile: &str) -> anyhow::Result<()> {
    let context = Context::new()
        .add_file("Dockerfile", dockerfile)?
        .add_sudoers()?
        .add_bin()?;
    build_image(context, tagname)?;
    run(tagname, r###"
        edgedb server install --version=1-alpha5
    "###).success();
    Ok(())
}

#[test_case("edbtest_bionic", &dock_ubuntu("bionic"))]
#[test_case("edbtest_xenial", &dock_ubuntu("xenial"))]
#[test_case("edbtest_focal", &dock_ubuntu("focal"))]
#[test_case("edbtest_centos7", &dock_centos(7))]
#[test_case("edbtest_centos8", &dock_centos(8))]
#[test_case("edbtest_buster", &dock_debian("buster"))]
#[test_case("edbtest_stretch", &dock_debian("stretch"))]
fn docker(tagname: &str, dockerfile: &str) -> anyhow::Result<()> {
    let context = Context::new()
        .add_file("Dockerfile", dockerfile)?
        .add_sudoers()?
        .add_bin()?;
    build_image(context, tagname)?;
    run_with_socket(tagname, r###"
        docker ps -q -f 'name=edgedb_test' | xargs -r docker container kill
        docker system prune --all --force
        docker volume list -q -f 'name=edgedb_test' | xargs -r docker volume rm
        mkdir /tmp/workdir
        cd /tmp/workdir

        edgedb server install --version=1-alpha6 --method=docker
        edgedb server init test1
        edgedb --wait-until-available=30s -Itest1 query '
            CREATE TYPE Type1 {
                CREATE PROPERTY prop1 -> str;
            }
        ' 'INSERT Type1 { prop1 := "value1" }'
        edgedb server upgrade test1 --to-nightly
        val=$(edgedb -Itest1 --wait-until-available=30s --tab-separated \
              query 'SELECT Type1 { prop1 }')
        test "$val" = "value1"
    "###).success();
    Ok(())
}
require_relative 'test_helper'

class GraphqlTest < Minitest::Test
  def entry_point
    "http://#{ENV['HOST']}:#{ENV['PORT']}/graphql"
  end

  def test_human
    query <<-GRAPHQL, id: '123'
      query ($id: String!) {
        human(id: $id) {
          id
          name
          appearsIn
          homePlanet
        }
      }
    GRAPHQL
    assert_ok
    human = data['human']
    assert_equal '1234', human['id']
    assert_equal 'Luke', human['name']
    assert_includes human['appearsIn'], 'NEW_HOPE'
    assert_equal 'Mars', human['homePlanet']
  end

  def test_create_human
    new_human = {
      name: 'hoge', appearsIn: 'JEDI', homePlanet: 'earth' 
    }
    query <<-GRAPHQL, human: new_human
      mutation($human: NewHuman!) {
        createHuman(newHuman: $human) {
          id
          name
          appearsIn
          homePlanet
        }
      }
    GRAPHQL
    assert_ok
    human = data['createHuman']
    assert_equal 'hoge', human['name']
    assert_includes human['appearsIn'], 'JEDI'
    assert_equal 'earth', human['homePlanet']
  end
end

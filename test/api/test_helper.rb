require 'minitest/autorun'
require 'net/http'
require 'uri'
require 'json'

class Minitest::Test
  def entry_point
    raise NotImplementedError
  end

  def before_request(req)
  end

  protected
  attr_reader :data

  def serialize_query(query, **variables)
    {
      query: query,
      variables: variables
    }.to_json
  end

  def post(data)
    uri = URI.parse entry_point
    http = Net::HTTP.new(uri.host, uri.port)
    req = Net::HTTP::Post.new(uri.request_uri)
    before_request req
    req.body = data
    req['Content-Type'] = 'application/json'
    res = http.request(req)
    @status = res.code
    @responce = JSON.parse(res.body) || {}
    @data = @responce['data']
  end

  def query(query, **variables)
    post(serialize_query(query, **variables))
  end

  def assert_ok
    assert_equal "200", @status
    assert_includes @responce.keys, 'data'
  end
end

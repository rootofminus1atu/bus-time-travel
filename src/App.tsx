import type { LatLngExpression } from "leaflet"
import { CircleMarker, MapContainer, Marker, Polyline, Popup, TileLayer } from "react-leaflet"
import axios from 'axios'
import { useQuery } from '@tanstack/react-query'
import L from 'leaflet'

// TODO: 
// - consider what happens when a vehicle stops operating
// - handle cases when theres less than 10 or 20 records (should be doable after flattening the output)
// - add time travel slider
// - add customization for the trail length, as well as frequency points (each 5 mins, each 2 mins, etc.)


type Location = {
  lat: number
  lon: number
  ts: string
  route: RouteInfo
  vehicle_id: string
}

function date(ts: string) {
  return new Date(Number(ts) * 1000).toLocaleTimeString()
}

function windowPairs<T>(arr: T[]) {
  return arr.slice(0, -1).map((item, i) => [item, arr[i + 1]])
}

type RouteInfo = {
  route_id: string,
  short_name: string,
  long_name: string
}

type Record = {
  ts: string
  locations: Location[]
}

type VehicleTrail = {
  route_id: string,
  vehicle_id: string,
  locations: Location[]
}

// TODO: refactir to be flat from the beginning
function buildVehicleTrails(records: Record[]): VehicleTrail[] {
  const map = new Map<string, VehicleTrail>()
  for (const record of records) {
    for (const loc of record.locations) {
      const key = `${loc.route.route_id}:${loc.vehicle_id}`
      if (!map.has(key)) {
        map.set(key, {
          route_id: loc.route.route_id,
          vehicle_id: loc.vehicle_id,
          locations: []
        })
      }
      map.get(key)!.locations.push(loc)
    }
  }
  return Array.from(map.values())
}

function getSegmentOpacity(idx: number, total: number) {
  return 0.3 + 0.7 * ((total - 1 - idx) / (total - 1))
}

async function fetchHistory(): Promise<Record[]> {
  const res = await axios.get<Record[]>('http://localhost:3000/history')
  const data = res.data.reverse()
  return data
}

// TODO: center at city c
const center: LatLngExpression = [51.897797, -8.441600]

function App() {
  const { data, isError, error } = useQuery({
    queryKey: ['history'],
    queryFn: fetchHistory
  })

  console.log(data)

  if (isError) {
    console.error(error)
  }

  const latest = data && data.length > 0 ? data[0] : null

  const tenLatest = data?.slice(0, 24) || []


  function getRouteColor(routeId: string, routeIds: string[]) {
    const palette = ['red', 'blue', 'green', 'orange', 'purple', 'brown']
    const idx = routeIds.indexOf(routeId)
    return palette[idx % palette.length]
  }

  const trails = buildVehicleTrails(tenLatest)
  const routeIds = Array.from(new Set(trails.map(t => t.route_id)))  


  function createRouteIcon(shortName: string, color: string) {
  return L.divIcon({
    className: "custom-marker",
    html: `<div style="
      background:${color};
      color:white;
      border-radius:50%;
      width:32px;
      height:32px;
      display:flex;
      align-items:center;
      justify-content:center;
      font-weight:bold;
      border:2px solid #fff;
      box-shadow:0 0 4px #0008;
      font-size:14px;
    ">${shortName}</div>`,
    iconSize: [32, 32],
    iconAnchor: [16, 16],
    popupAnchor: [0, -16],
  })
}

  return (
    <div>
      <MapContainer center={center} zoom={14} style={{ height: "100%", width: "100%" }}>
        <TileLayer
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        {
          latest && latest.locations.map(loc => (
            <>
              <Marker 
                key={loc.vehicle_id} 
                position={[loc.lat, loc.lon]}
                icon={createRouteIcon(loc.route.short_name, getRouteColor(loc.route.route_id, routeIds))}
              >
                <Popup>
                  Vehicle {loc.vehicle_id}<br />
                  Route: {loc.route.short_name}<br />
                  Time: {date(loc.ts)}
                </Popup>
              </Marker>
            </>
          ))
        }
        {trails.map(trail => {
          const segments = windowPairs(trail.locations)
          return segments.map(([a, b], idx) => (
            <Polyline
              key={`${trail.route_id}:${trail.vehicle_id}:${idx}`}
              positions={[[a.lat, a.lon], [b.lat, b.lon]]}
              color={getRouteColor(trail.route_id, routeIds)}
              opacity={getSegmentOpacity(idx, segments.length)}
              weight={5}
            />
          ))
        })}
        {trails.map(trail => {
          const sorted = [...trail.locations].sort((a, b) => Number(b.ts) - Number(a.ts))
          const now = sorted[0]

          const nowTs = Number(now.ts)
          const fiveMinAgo = sorted.find(loc => nowTs - Number(loc.ts) >= 5 * 60)
          const tenMinAgo = sorted.find(loc => nowTs - Number(loc.ts) >= 10 * 60)

          return (
            <>
              {fiveMinAgo && (
                <CircleMarker
                  center={[fiveMinAgo.lat, fiveMinAgo.lon]}
                  radius={10}
                  color={getRouteColor(trail.route_id, routeIds)}
                  fillColor={getRouteColor(trail.route_id, routeIds)}
                  fillOpacity={0.6}
                  stroke={false}
                >
                  <Popup>
                    5 mins ago<br />
                    {date(fiveMinAgo.ts)}
                  </Popup>
                </CircleMarker>
              )}
              {tenMinAgo && (
                <CircleMarker
                  center={[tenMinAgo.lat, tenMinAgo.lon]}
                  radius={10}
                  color={getRouteColor(trail.route_id, routeIds)}
                  fillColor={getRouteColor(trail.route_id, routeIds)}
                  fillOpacity={0.3}
                  stroke={false}
                >
                  <Popup>
                    10 mins ago<br />
                    {date(tenMinAgo.ts)}
                  </Popup>
                </CircleMarker>
              )}
            </>
          )
        })}
      </MapContainer>

    </div>
  )
}

export default App

using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FB RID: 251
	[HandlerCategory("vvTrade"), HandlerName("Номер бара входа в позицию (от инструм.)")]
	public class EntryBarNumFromSrc : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600075C RID: 1884 RVA: 0x0002091C File Offset: 0x0001EB1C
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionClosed = sec.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed == null)
			{
				return 0.0;
			}
			return (double)lastPositionClosed.get_EntryBarNum();
		}
	}
}

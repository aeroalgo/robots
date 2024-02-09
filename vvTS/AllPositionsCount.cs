using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000C9 RID: 201
	[HandlerCategory("vvTrade"), HandlerName("Количество всех позиций")]
	public class AllPositionsCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006DF RID: 1759 RVA: 0x0001EC58 File Offset: 0x0001CE58
		public double Execute(ISecurity sec, int barNum)
		{
			int activePositionCount = sec.get_Positions().get_ActivePositionCount();
			if (activePositionCount < 1)
			{
				return 0.0;
			}
			IPositionsList positions = sec.get_Positions();
			double num = 0.0;
			foreach (IPosition arg_3A_0 in positions)
			{
				num += 1.0;
			}
			return num;
		}
	}
}

using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C2 RID: 194
	[HandlerCategory("vvTrade"), HandlerName("Текущий Bid")]
	public class CurrentBid : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006D1 RID: 1745 RVA: 0x0001E934 File Offset: 0x0001CB34
		public IList<double> Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			double num = (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_Bid().HasValue ? securityRt.get_FinInfo().get_Bid().Value : 0.0);
			double[] array = new double[sec.get_Bars().Count];
			for (int i = 0; i < array.Length; i++)
			{
				array[i] = num;
			}
			return array;
		}
	}
}

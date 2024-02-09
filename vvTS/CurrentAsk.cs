using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C4 RID: 196
	[HandlerCategory("vvTrade"), HandlerName("Текущий Ask")]
	public class CurrentAsk : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006D5 RID: 1749 RVA: 0x0001EA18 File Offset: 0x0001CC18
		public IList<double> Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			double num = (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_Ask().HasValue ? securityRt.get_FinInfo().get_Ask().Value : 0.0);
			double[] array = new double[sec.get_Bars().Count];
			for (int i = 0; i < array.Length; i++)
			{
				array[i] = num;
			}
			return array;
		}
	}
}

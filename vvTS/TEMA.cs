using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000163 RID: 355
	[HandlerCategory("vvAverages"), HandlerName("TEMA")]
	public class TEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B3C RID: 2876 RVA: 0x0002E1E1 File Offset: 0x0002C3E1
		public IList<double> Execute(IList<double> src)
		{
			return TEMA.GenTEMA(src, this.TemaPeriod);
		}

		// Token: 0x06000B3B RID: 2875 RVA: 0x0002E168 File Offset: 0x0002C368
		public static IList<double> GenTEMA(IList<double> src, int period)
		{
			IList<double> list = EMA.GenEMA(src, period);
			IList<double> list2 = EMA.GenEMA(list, period);
			IList<double> list3 = EMA.GenEMA(list2, period);
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 3.0 * list[i] - 3.0 * list2[i] + list3[i];
			}
			return array;
		}

		// Token: 0x170003B4 RID: 948
		public IContext Context
		{
			// Token: 0x06000B3D RID: 2877 RVA: 0x0002E1EF File Offset: 0x0002C3EF
			get;
			// Token: 0x06000B3E RID: 2878 RVA: 0x0002E1F7 File Offset: 0x0002C3F7
			set;
		}

		// Token: 0x170003B3 RID: 947
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int TemaPeriod
		{
			// Token: 0x06000B39 RID: 2873 RVA: 0x0002E157 File Offset: 0x0002C357
			get;
			// Token: 0x06000B3A RID: 2874 RVA: 0x0002E15F File Offset: 0x0002C35F
			set;
		}
	}
}

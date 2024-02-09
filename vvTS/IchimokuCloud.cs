using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000081 RID: 129
	[HandlerCategory("vvIchimoku"), HandlerName("IchimokuCloud")]
	public class IchimokuCloud : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000489 RID: 1161 RVA: 0x0001771C File Offset: 0x0001591C
		public IList<double> Execute(IList<double> senkouA, IList<double> senkouB)
		{
			if (senkouA.Count != senkouB.Count)
			{
				return null;
			}
			int count = senkouA.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (((i & 1) == 1) ? senkouA[i] : senkouB[i]);
			}
			return array;
		}
	}
}

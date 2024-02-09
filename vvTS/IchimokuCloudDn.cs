using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000083 RID: 131
	[HandlerCategory("vvIchimoku"), HandlerName("CloudDn")]
	public class IchimokuCloudDn : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600048D RID: 1165 RVA: 0x000177F0 File Offset: 0x000159F0
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
				if (senkouA[i] < senkouB[i])
				{
					array[i] = (((i & 1) == 1) ? senkouA[i] : senkouB[i]);
				}
				else
				{
					array[i] = 0.0;
				}
			}
			return array;
		}
	}
}

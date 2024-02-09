using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000082 RID: 130
	[HandlerCategory("vvIchimoku"), HandlerName("CloudUp")]
	public class IchimokuCloudUp : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600048B RID: 1163 RVA: 0x00017778 File Offset: 0x00015978
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
				if (senkouA[i] >= senkouB[i])
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
